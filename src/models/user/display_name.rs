use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

pub struct DisplayName(String);

impl TryFrom<String> for DisplayName {
    type Error = DisplayNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim().to_string();

        if value.is_empty() {
            return Err(DisplayNameError::Empty);
        }

        if value.graphemes(true).count() > 64 {
            return Err(DisplayNameError::LengthExceeded);
        }

        Ok(DisplayName(value))
    }
}

impl AsRef<str> for DisplayName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum DisplayNameError {
    #[error("Display name cannot be empty")]
    Empty,

    #[error("Display name is too long")]
    LengthExceeded,
}
