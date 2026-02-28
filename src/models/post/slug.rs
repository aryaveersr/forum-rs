use slug::slugify;

use crate::{models::post::title::Title, utils};

pub struct Slug {
    base: String,
    random: String,
}

impl From<&Title> for Slug {
    fn from(value: &Title) -> Self {
        Slug {
            base: slugify(value.as_ref()),
            random: utils::random_string(),
        }
    }
}

impl Slug {
    pub fn randomize(&mut self) {
        self.random = utils::random_string();
    }
}

impl ToString for Slug {
    fn to_string(&self) -> String {
        format!("{}-{}", self.base, self.random)
    }
}
