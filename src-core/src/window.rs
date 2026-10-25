use std::process::Command;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

pub struct WindowManager;

impl WindowManager {
    pub fn spawn_window(url: &str, config: &WindowConfig) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            // Try Edge App Mode (Chromium/WebView2 runtime)
            let edge_paths = [
                "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
                "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
            ];

            let mut launched = false;
            for path in &edge_paths {
                if std::path::Path::new(path).exists() {
                    let mut cmd = Command::new(path);
                    cmd.arg(format!("--app={}", url))
                        .arg(format!("--window-size={},{}", config.width, config.height))
                        .arg("--no-first-run")
                        .arg("--no-default-browser-check");
                    if cmd.spawn().is_ok() {
                        launched = true;
                        break;
                    }
                }
            }

            if !launched {
                // Fallback to default browser
                Command::new("cmd")
                    .args(["/C", "start", url])
                    .spawn()?;
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix fallback
            Command::new("xdg-open").arg(url).spawn()?;
        }

        Ok(())
    }
}
