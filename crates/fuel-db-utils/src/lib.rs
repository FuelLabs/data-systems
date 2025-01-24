pub(crate) mod cli;
pub mod config;

pub fn generate_random_api_key() -> String {
    use fake::rand::{self, distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| c.is_ascii_alphabetic())
        .take(12)
        .map(char::from)
        .collect()
}
