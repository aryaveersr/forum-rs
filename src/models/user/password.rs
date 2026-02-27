use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

pub struct Password(String);

impl TryFrom<String> for Password {
    type Error = PasswordError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(PasswordError::Empty);
        }

        if value.graphemes(true).count() < 8 {
            return Err(PasswordError::LengthTooShort);
        }

        if value.graphemes(true).count() > 128 {
            return Err(PasswordError::LengthExceeded);
        }

        Ok(Password(value))
    }
}

impl Password {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Username cannot be empty")]
    Empty,

    #[error("Username is too short")]
    LengthTooShort,

    #[error("Username is too long")]
    LengthExceeded,
}
