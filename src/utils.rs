use uuid::Uuid;

pub fn random_string() -> String {
    Uuid::new_v4()
        .to_string()
        .split('-')
        .next_back()
        .unwrap()
        .to_owned()
}
