use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum LanternError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Java error: {0}")]
    Java(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Hash mismatch for {file}: expected {expected}, got {actual}")]
    HashMismatch {
        file: String,
        expected: String,
        actual: String,
    },

    #[error("{0}")]
    Other(String),
}

impl Serialize for LanternError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("LanternError", 2)?;
        state.serialize_field("kind", &self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl LanternError {
    fn kind(&self) -> &'static str {
        match self {
            Self::Network(_) => "network",
            Self::Io(_) => "io",
            Self::Zip(_) => "zip",
            Self::Json(_) => "json",
            Self::Auth(_) => "auth",
            Self::Instance(_) => "instance",
            Self::Java(_) => "java",
            Self::Launch(_) => "launch",
            Self::HashMismatch { .. } => "hash_mismatch",
            Self::Other(_) => "other",
        }
    }
}
