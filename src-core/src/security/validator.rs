use std::collections::HashSet;

pub struct OriginValidator {
    allowed_origins: HashSet<String>,
}

impl OriginValidator {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        let set = allowed_origins
            .into_iter()
            .map(|origin| origin.trim_end_matches('/').to_ascii_lowercase())
            .collect();
        Self { allowed_origins: set }
    }

    #[inline]
    pub fn is_allowed(&self, origin: Option<&str>) -> bool {
        match origin {
            Some(o) => {
                let trimmed = o.trim_end_matches('/');
                if self.allowed_origins.contains(trimmed) {
                    return true;
                }
                self.allowed_origins.contains(&trimmed.to_ascii_lowercase())
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_validation_normalized_and_case_insensitive() {
        let validator = OriginValidator::new(vec![
            "http://127.0.0.1:5173/".to_string(),
            "http://localhost:5173".to_string(),
        ]);

        assert!(validator.is_allowed(Some("http://127.0.0.1:5173")));
        assert!(validator.is_allowed(Some("http://127.0.0.1:5173/")));
        assert!(validator.is_allowed(Some("HTTP://127.0.0.1:5173")));
        assert!(validator.is_allowed(Some("http://LOCALHOST:5173/")));
        assert!(!validator.is_allowed(Some("http://127.0.0.1:5174")));
        assert!(!validator.is_allowed(Some("http://evil.com")));
        assert!(!validator.is_allowed(None));
    }
}
