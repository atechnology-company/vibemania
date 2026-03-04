//! Windows AI backend via Windows ML / Phi-4 mini (Copilot+ PCs, Windows 11).
//!
//! Uses the Windows AI API (WinRT) to run Phi-4 mini on-device.
//! Requires: Windows 11 24H2+, Copilot+ PC (NPU), `windows` crate.
//!
//! WinRT path:
//!   Windows.AI.MachineLearning or
//!   Microsoft.Windows.AI.Inference (preview, Phi-4 mini)
//!
//! On non-Windows targets this module is a no-op.

use anyhow::{bail, Result};

/// Check if Windows AI is available
pub fn is_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check Windows version and AI API availability
        // Windows.AI.Inference requires Windows 11 24H2 (build 26100+)
        use std::process::Command;
        let out = Command::new("powershell")
            .args(["-Command", "[System.Environment]::OSVersion.Version.Build"])
            .output()
            .ok();
        if let Some(out) = out {
            let build: u32 = String::from_utf8_lossy(&out.stdout)
                .trim().parse().unwrap_or(0);
            // 26100 = Windows 11 24H2
            return build >= 26100;
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    false
}

/// Run a prompt through Windows AI (Phi-4 mini)
pub async fn run(_prompt: &str) -> Result<String> {
    #[cfg(target_os = "windows")]
    {
        // Use PowerShell + Windows.AI API via COM/WinRT
        // In production: use `windows-rs` crate with Windows::AI::Inference bindings
        use tokio::process::Command;

        let ps_code = format!(
            r#"
Add-Type -AssemblyName "Windows.AI.MachineLearning, Version=10.0.0.0, Culture=neutral"
$session = [Windows.AI.Inference.LanguageModel]::CreateAsync().GetAwaiter().GetResult()
$response = $session.GenerateResponseAsync("{prompt}").GetAwaiter().GetResult()
Write-Output $response.Response
"#,
            prompt = prompt.replace('"', "'")
        );

        let out = Command::new("powershell")
            .args(["-Command", &ps_code])
            .output()
            .await?;

        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            bail!(
                "Windows AI call failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )
        }
    }
    #[cfg(not(target_os = "windows"))]
    bail!("Windows AI not available on this platform")
}
