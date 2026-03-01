use serde::Deserialize;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
pub struct Title(String);

impl TryFrom<String> for Title {
    type Error = TitleError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(TitleError::Empty);
        }

        if value.graphemes(true).count() > 128 {
            return Err(TitleError::LengthExceeded);
        }

        Ok(Title(value))
    }
}

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum TitleError {
    #[error("Title cannot be empty")]
    Empty,

    #[error("Title is too long")]
    LengthExceeded,
}
