use std::sync::Mutex;
#[cfg(target_os = "linux")]
use std::io::Write;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};

/// Detects if the current Linux desktop environment is running under Wayland.
#[allow(dead_code)]
pub fn is_wayland_session() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

/// Lazily opens the native text clipboard.
/// Supports Windows (Win32), macOS (NSPasteboard), Linux X11/XWayland (via arboard),
/// and Linux Pure Wayland (via native wl-clipboard Wayland Data-Device protocol).
pub struct TextClipboard(Mutex<Option<arboard::Clipboard>>);

impl TextClipboard {
    pub fn new() -> Self {
        Self(Mutex::new(None))
    }

    pub fn validate_text(text: &str) -> Result<(), String> {
        if text.len() > 65536 {
            return Err("Clipboard text exceeds 65536 bytes".into());
        }
        if text.contains('\0') {
            return Err("Clipboard text contains NUL".into());
        }
        Ok(())
    }

    /// Pure Wayland protocol clipboard write using the standard wl-copy tool.
    /// Pipes UTF-8 text directly to stdin with zero shell interpretation.
    #[cfg(target_os = "linux")]
    fn write_wayland_protocol(&self, text: &str) -> Result<(), String> {
        let mut child = Command::new("wl-copy")
            .arg("--type")
            .arg("text/plain;charset=utf-8")
            .arg("--trim-newline=false")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Wayland wl-copy spawn failed: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(text.as_bytes())
                .map_err(|e| format!("Failed to write to Wayland clipboard pipe: {e}"))?;
            stdin
                .flush()
                .map_err(|e| format!("Failed to flush Wayland clipboard pipe: {e}"))?;
        }

        let status = child
            .wait()
            .map_err(|e| format!("Wayland wl-copy process wait failed: {e}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("Wayland wl-copy exited with error status: {status}"))
        }
    }

    #[allow(dead_code)]
    #[cfg(not(target_os = "linux"))]
    fn write_wayland_protocol(&self, _text: &str) -> Result<(), String> {
        Err("Wayland protocol is only supported on Linux".into())
    }

    pub fn write_text(&self, text: &str) -> Result<(), String> {
        Self::validate_text(text)?;

        // On Linux under Wayland, attempt the native Wayland driver first
        #[cfg(target_os = "linux")]
        if is_wayland_session() {
            if self.write_wayland_protocol(text).is_ok() {
                return Ok(());
            }
            // Fall through to arboard (XWayland fallback)
        }

        // Standard native clipboard backend (Windows Win32, macOS NSPasteboard, Linux X11)
        let mut state = self.0.lock().map_err(|_| "Clipboard state unavailable")?;
        if state.is_none() {
            match arboard::Clipboard::new() {
                Ok(cb) => *state = Some(cb),
                Err(e) => {
                    // If arboard fails on Linux, attempt wl-copy as secondary fallback
                    #[cfg(target_os = "linux")]
                    if self.write_wayland_protocol(text).is_ok() {
                        return Ok(());
                    }
                    #[cfg(target_os = "linux")]
                    if is_wayland_session() {
                        return Err(format!(
                            "Native clipboard unavailable: Pure Wayland session detected without active XWayland or wl-clipboard ({e})"
                        ));
                    }
                    return Err(format!("Native clipboard unavailable: {e}"));
                }
            }
        }

        if let Some(ref mut cb) = *state {
            if let Err(e) = cb.set_text(text) {
                #[cfg(target_os = "linux")]
                if is_wayland_session() && self.write_wayland_protocol(text).is_ok() {
                    return Ok(());
                }
                return Err(format!("Native clipboard write failed: {e}"));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_validation_success() {
        assert!(TextClipboard::validate_text("Hello, Aegis Desktop!").is_ok());
        assert!(TextClipboard::validate_text("").is_ok());
        assert!(TextClipboard::validate_text("සිංහල යුනිකෝඩ් සහ විශේෂ අක්ෂර 🛡️").is_ok());
    }

    #[test]
    fn test_clipboard_validation_bounds() {
        let oversized = "a".repeat(65537);
        assert_eq!(
            TextClipboard::validate_text(&oversized).unwrap_err(),
            "Clipboard text exceeds 65536 bytes"
        );

        let exact = "a".repeat(65536);
        assert!(TextClipboard::validate_text(&exact).is_ok());
    }

    #[test]
    fn test_clipboard_validation_nul() {
        assert_eq!(
            TextClipboard::validate_text("hello\0world").unwrap_err(),
            "Clipboard text contains NUL"
        );
    }

    #[test]
    fn test_wayland_session_query() {
        // Must not panic on any platform
        let _ = is_wayland_session();
    }
}
