use serde::Deserialize;
use slug::slugify;
use thiserror::Error;

use crate::{domain::post::title::Title, utils};

#[derive(Deserialize, Debug)]
#[serde(try_from = "String")]
pub struct Slug(String);

impl From<&Title> for Slug {
    fn from(value: &Title) -> Self {
        Slug(format!(
            "{}-{}",
            slugify(value.as_ref()),
            utils::random_string()
        ))
    }
}

impl TryFrom<String> for Slug {
    type Error = SlugError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(SlugError::Empty);
        }

        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(SlugError::InvalidCharacter);
        }

        Ok(Slug(value))
    }
}

impl Slug {
    pub fn randomize(&mut self) {
        self.0 = match self.0.rsplit_once('-') {
            Some(prefix) => format!("{}-{}", prefix.0, utils::random_string()),
            None => format!("{}-{}", self.0, utils::random_string()),
        };
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SlugError {
    #[error("Slug cannot be empty")]
    Empty,

    #[error("Slug contains invalid character")]
    InvalidCharacter,
}
