use std::sync::Mutex;

/// Lazily opens the native text clipboard. Retention is necessary for X11 ownership.
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

    pub fn write_text(&self, text: &str) -> Result<(), String> {
        Self::validate_text(text)?;
        let mut state = self.0.lock().map_err(|_| "Clipboard state unavailable")?;
        if state.is_none() {
            *state = Some(arboard::Clipboard::new().map_err(|_| "Native clipboard unavailable")?);
        }
        state
            .as_mut()
            .ok_or("Native clipboard unavailable")?
            .set_text(text)
            .map_err(|_| "Native clipboard write failed".into())
    }
}
