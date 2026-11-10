use serde::Serializer;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Message(String),
    #[error("base de données : {0}")]
    Db(#[from] rusqlite::Error),
}

impl AppError {
    pub fn message(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
