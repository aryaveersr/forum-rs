use std::fmt::{self, Display, Formatter};

use slug::slugify;
use thiserror::Error;

use crate::{domain::post::title::Title, utils};

pub struct Slug {
    base: String,
    random: Option<String>,
}

impl From<&Title> for Slug {
    fn from(value: &Title) -> Self {
        Slug {
            base: slugify(value.as_ref()),
            random: Some(utils::random_string()),
        }
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

        Ok(Slug {
            base: value,
            random: None,
        })
    }
}

impl Slug {
    pub fn randomize(&mut self) {
        self.random = Some(utils::random_string());
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.base)?;

        if let Some(random) = &self.random {
            write!(f, "-{random}")?;
        }

        Ok(())
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
