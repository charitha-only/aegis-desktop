pub struct OriginValidator {
    allowed_origins: Vec<String>,
}

impl OriginValidator {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins }
    }

    pub fn is_allowed(&self, origin: Option<&str>) -> bool {
        match origin {
            Some(o) => {
                let trimmed = o.trim_end_matches('/');
                self.allowed_origins.iter().any(|allowed| allowed.trim_end_matches('/') == trimmed)
            }
            None => false,
        }
    }
}
