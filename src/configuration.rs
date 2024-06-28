use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use std::fs::read_to_string;
use toml;

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format! {
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            }),
        }
    }
}

pub fn get_config() -> Result<Settings, anyhow::Error> {
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    let mut settings: Settings = toml::from_str(&read_to_string("configuration.toml")?)?;

    match environment {
        Environment::Local => {
            settings.application.host = "127.0.0.1".to_string();
            settings.database.require_ssl = false;
        }
        Environment::Production => {
            settings.database.require_ssl = true;
            settings.database.username = std::env::var("APP_DATABASE__USERNAME")
                .expect("Failed to parse APP_DATABASE__USERNAME");
            settings.database.password = Secret::new(
                std::env::var("APP_DATABASE__PASSWORD")
                    .expect("Failed to parse APP_DATABASE__PASSWORD"),
            );
            settings.application.host = std::env::var("APP_DATABASE__HOSTNAME")
                .expect("Failed to parse APP_DATABASE__HOSTNAME");
            settings.application.port = std::env::var("APP_DATABASE__PORT")
                .expect("Failed to parse APP_DATABASE__PORT")
                .parse()
                .expect("Unable to parse port into u16");
            settings.database.database_name = std::env::var("APP_DATABASE__DATABASE_NAME")
                .expect("Failed to parse APP_DATABASE__DATABASE_NAME");
        }
    }

    Ok(settings)
}
