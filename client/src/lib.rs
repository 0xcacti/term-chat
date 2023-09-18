use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
        }
    }
}

impl Config {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }
}
