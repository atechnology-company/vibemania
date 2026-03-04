//! ACP (Agent Client Protocol) integration for Subspace.
//! Spawns claude-agent-acp as a subprocess and communicates via structured JSON-RPC.
//! Auto-approves all permission requests (bypass mode).

use agent_client_protocol as acp;
use acp::Agent as _; // bring trait methods into scope
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

/// Collected output from an ACP session
#[derive(Debug, Clone, Default)]
pub struct SessionOutput {
    pub text: String,
    pub tool_calls: Vec<ToolCallRecord>,
    pub files_modified: Vec<String>,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub status: String,
}

/// ACP Client that auto-approves everything (bypass permissions)
struct SubspaceClient {
    output: Arc<Mutex<SessionOutput>>,
}

#[async_trait::async_trait(?Send)]
impl acp::Client for SubspaceClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        // Auto-approve everything — pick the first "allow always" or "allow once" option
        let option_id = args
            .options
            .iter()
            .find(|o| matches!(o.kind, acp::PermissionOptionKind::AllowAlways))
            .or_else(|| args.options.iter().find(|o| matches!(o.kind, acp::PermissionOptionKind::AllowOnce)))
            .or_else(|| args.options.first())
            .map(|o| o.option_id.clone());

        match option_id {
            Some(id) => Ok(acp::RequestPermissionResponse::new(
                acp::RequestPermissionOutcome::Selected(
                    acp::SelectedPermissionOutcome::new(id),
                ),
            )),
            None => Ok(acp::RequestPermissionResponse::new(
                acp::RequestPermissionOutcome::Cancelled,
            )),
        }
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<()> {
        let mut output = self.output.lock().await;
        match args.update {
            acp::SessionUpdate::AgentMessageChunk(chunk) => {
                match chunk.content {
                    acp::ContentBlock::Text(text) => {
                        output.text.push_str(&text.text);
                    }
                    _ => {}
                }
            }
            acp::SessionUpdate::ToolCall(tc) => {
                output.tool_calls.push(ToolCallRecord {
                    tool_name: tc.title.clone(),
                    status: "started".to_string(),
                });
            }
            acp::SessionUpdate::ToolCallUpdate(tc) => {
                if let Some(status) = &tc.fields.status {
                    if let Some(last) = output.tool_calls.last_mut() {
                        last.status = format!("{:?}", status);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn write_text_file(
        &self,
        args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        let mut output = self.output.lock().await;
        output.files_modified.push(args.path.to_string_lossy().to_string());

        if let Err(_e) = std::fs::write(&args.path, &args.content) {
            return Err(acp::Error::internal_error());
        }
        Ok(acp::WriteTextFileResponse::new())
    }

    async fn read_text_file(
        &self,
        args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        match std::fs::read_to_string(&args.path) {
            Ok(content) => Ok(acp::ReadTextFileResponse::new(content)),
            Err(_e) => Err(acp::Error::internal_error()),
        }
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Ok(())
    }
}

/// Find the claude-agent-acp binary
fn find_acp_binary() -> Result<PathBuf> {
    if let Ok(output) = std::process::Command::new("which").arg("claude-agent-acp").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let npm_path = PathBuf::from(&home).join(".npm-global/bin/claude-agent-acp");
    if npm_path.exists() {
        return Ok(npm_path);
    }

    anyhow::bail!(
        "claude-agent-acp not found. Install it:\n  npm install -g @zed-industries/claude-agent-acp"
    );
}

/// Run a prompt through ACP with full permission bypass
pub async fn run_prompt(cwd: &Path, prompt: &str) -> Result<SessionOutput> {
    let acp_binary = find_acp_binary()?;
    let output = Arc::new(Mutex::new(SessionOutput::default()));

    let mut child = tokio::process::Command::new(&acp_binary)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Failed to spawn claude-agent-acp")?;

    let stdin = child.stdin.take().unwrap().compat_write();
    let stdout = child.stdout.take().unwrap().compat();

    let session_output = output.clone();
    let cwd = cwd.to_path_buf();
    let prompt = prompt.to_string();

    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            let client = SubspaceClient {
                output: session_output.clone(),
            };

            let (conn, handle_io) =
                acp::ClientSideConnection::new(client, stdin, stdout, |fut| {
                    tokio::task::spawn_local(fut);
                });

            tokio::task::spawn_local(handle_io);

            // Initialize
            let init_req = acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                .client_info(acp::Implementation::new("subspace", env!("CARGO_PKG_VERSION")));

            conn.initialize(init_req).await?;

            // Create session
            let session = conn
                .new_session(acp::NewSessionRequest::new(&cwd))
                .await?;

            // Send prompt
            let prompt_content = acp::ContentBlock::Text(acp::TextContent::new(&prompt));
            let result = conn
                .prompt(acp::PromptRequest::new(
                    session.session_id.clone(),
                    vec![prompt_content],
                ))
                .await;

            match result {
                Ok(response) => {
                    let mut out = session_output.lock().await;
                    out.stop_reason = Some(format!("{:?}", response.stop_reason));
                }
                Err(e) => {
                    let mut out = session_output.lock().await;
                    out.stop_reason = Some(format!("error: {}", e));
                }
            }

            drop(child);
            Ok::<(), anyhow::Error>(())
        })
        .await?;

    let result = output.lock().await.clone();
    Ok(result)
}
