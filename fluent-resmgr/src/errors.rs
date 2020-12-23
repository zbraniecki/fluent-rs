use std::error::Error;

#[derive(Debug)]
pub enum ResourceManagerError {
    MissingResource(String),
}

impl std::fmt::Display for ResourceManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingResource(res_id) => write!(f, "Missing resource: {}", res_id),
        }
    }
}

impl Error for ResourceManagerError {}
