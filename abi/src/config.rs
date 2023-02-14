use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Config {
    pub db: DbConfig,
    pub server: ServerConfig,
}

impl Config {
    pub fn load(filename: impl AsRef<Path>) -> Result<Self, Error> {
        let f: String = std::fs::read_to_string(filename).map_err(|_| Error::ConfigReadError)?;
        serde_yaml::from_str(&f).map_err(|_| Error::ConfigReadError)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    #[serde(default = "default_pool_size")]
    pub max_connections: u32,
}

impl DbConfig {
    pub fn server_url(&self) -> String {
        if self.password.is_empty() {
            format!("postgres://{}@{}:{}", self.user, self.host, self.port)
        } else {
            format!(
                "postgres://{}:{}@{}:{}",
                self.user, self.password, self.host, self.port
            )
        }
    }

    pub fn db_url(&self) -> String {
        format!("{}/{}", self.server_url(), self.dbname)
    }
}

fn default_pool_size() -> u32 {
    5
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_should_loaded() {
        let filename = String::from("../service/fixtures/config.yml");
        assert_eq!(
            Config::load(filename),
            Ok(Config {
                db: DbConfig {
                    host: String::from("localhost"),
                    port: 5432,
                    user: String::from("postgres"),
                    password: String::from("postgres"),
                    dbname: String::from("reservation"),
                    max_connections: 5,
                },
                server: ServerConfig {
                    host: String::from("0.0.0.0"),
                    port: 50051,
                },
            })
        );
    }
}
