use std::collections::HashSet;

pub struct OriginValidator {
    allowed_origins: HashSet<String>,
}

impl OriginValidator {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        let set = allowed_origins
            .into_iter()
            .map(|origin| origin.trim_end_matches('/').to_string())
            .collect();
        Self { allowed_origins: set }
    }

    #[inline]
    pub fn is_allowed(&self, origin: Option<&str>) -> bool {
        match origin {
            Some(o) => {
                let trimmed = o.trim_end_matches('/');
                self.allowed_origins.contains(trimmed)
            }
            None => false,
        }
    }
}
