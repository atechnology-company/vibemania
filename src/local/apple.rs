//! Apple Intelligence backend via FoundationModels (macOS 15+).
//!
//! Uses ObjC FFI via the `objc2` crate to call the Swift FoundationModels
//! framework through a thin ObjC bridge.
//!
//! FoundationModels API (macOS 15+):
//!   import FoundationModels
//!   let session = LanguageModelSession()
//!   let response = try await session.respond(to: prompt)
//!
//! The bridge is compiled as a separate dylib (build.rs handles this on macOS).
//! On non-macOS targets this module is a no-op.

use anyhow::{bail, Result};

/// Check if Apple Intelligence is available on this machine
pub fn is_available() -> bool {
    #[cfg(target_os = "macos")]
    {
        // Check macOS version >= 15.0 (FoundationModels requires it)
        // and that the FoundationModels framework is loadable
        use std::process::Command;
        let out = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok();
        if let Some(out) = out {
            let v = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<u32> = v.trim().split('.').filter_map(|p| p.parse().ok()).collect();
            if parts.first().copied().unwrap_or(0) >= 15 {
                // Check framework exists
                return std::path::Path::new(
                    "/System/Library/Frameworks/FoundationModels.framework"
                ).exists();
            }
        }
        false
    }
    #[cfg(not(target_os = "macos"))]
    false
}

/// Run a prompt through Apple Intelligence (FoundationModels)
pub async fn run(prompt: &str) -> Result<String> {
    #[cfg(target_os = "macos")]
    {
        // Spawn a tiny Swift one-liner via `swift -e` (available on macOS with Xcode CLI tools)
        // This avoids needing a full ObjC bridge compile at build time.
        // For production use, replace with objc2 FFI + pre-compiled bridge dylib.
        use tokio::process::Command;

        let swift_code = format!(
            r#"
import FoundationModels
import Foundation

let session = LanguageModelSession()
let response = try await session.respond(to: "{prompt}")
print(response.content)
"#,
            prompt = prompt.replace('"', "'")
        );

        let out = Command::new("swift")
            .args(["-e", &swift_code])
            .output()
            .await?;

        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            bail!(
                "Apple Intelligence call failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )
        }
    }
    #[cfg(not(target_os = "macos"))]
    bail!("Apple Intelligence not available on this platform")
}
