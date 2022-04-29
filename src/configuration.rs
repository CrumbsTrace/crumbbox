use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
    pub storage_path: String,
}

impl Settings {
    pub fn get_configuration() -> Result<Self, ConfigError> {
        let base_path = std::env::current_dir().expect("Failed to determine current directory");
        let configuration_path = base_path.join("configuration");

        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| String::from("local"))
            .try_into()
            .expect("Failed to parse environment");

        let config = Config::builder()
            .add_source(File::from(configuration_path.join("default")))
            .add_source(File::from(configuration_path.join(environment.as_str())).required(true))
            .add_source(config::Environment::with_prefix("app"))
            .build()?;

        config.try_deserialize()
    }
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a valid environment. Use local or production",
                other
            )),
        }
    }
}
