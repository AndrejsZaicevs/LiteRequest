use serde::Serialize;

#[derive(Debug)]
pub enum LiteRequestError {
    Db(rusqlite::Error),
    Http(String),
    Import(String),
    Validation(String),
    LockPoisoned(String),
    Internal(String),
}

impl std::fmt::Display for LiteRequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Db(e) => write!(f, "Database error: {e}"),
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Import(e) => write!(f, "Import error: {e}"),
            Self::Validation(e) => write!(f, "Validation error: {e}"),
            Self::LockPoisoned(e) => write!(f, "Lock poisoned: {e}"),
            Self::Internal(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LiteRequestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Db(e) => Some(e),
            _ => None,
        }
    }
}

impl Serialize for LiteRequestError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<rusqlite::Error> for LiteRequestError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

impl From<reqwest::Error> for LiteRequestError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e.to_string())
    }
}

impl From<String> for LiteRequestError {
    fn from(s: String) -> Self {
        Self::Internal(s)
    }
}

impl From<&str> for LiteRequestError {
    fn from(s: &str) -> Self {
        Self::Internal(s.to_string())
    }
}
