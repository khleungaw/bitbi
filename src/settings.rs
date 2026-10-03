use config::Config;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Settings
{
    pub addr: String,
    pub port: u16,
}

impl Settings
{
    pub fn new(app_env: &str) -> Result<Settings, config::ConfigError>
    {
        Config::builder()
            .add_source(config::File::with_name("settings/settings"))
            .add_source(config::File::with_name(&format!(
                "settings/settings_{}",
                app_env
            )))
            .build()
            .unwrap()
            .try_deserialize()
    }
}
