use serde::Deserialize;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
pub struct Username(String);

impl TryFrom<String> for Username {
    type Error = UsernameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(UsernameError::Empty);
        }

        if value.graphemes(true).count() > 64 {
            return Err(UsernameError::LengthExceeded);
        }

        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(UsernameError::InvalidCharacters);
        }

        Ok(Username(value))
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum UsernameError {
    #[error("Username cannot be empty")]
    Empty,

    #[error("Username is too long")]
    LengthExceeded,

    #[error("Username contains invalid characters")]
    InvalidCharacters,
}
