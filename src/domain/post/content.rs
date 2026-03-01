use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct Content(String);

impl TryFrom<String> for Content {
    type Error = ContentError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(ContentError::Empty);
        }

        Ok(Content(value))
    }
}

impl AsRef<str> for Content {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ContentError {
    #[error("Content cannot be empty")]
    Empty,
}
