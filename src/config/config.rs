use serde::Deserialize;
use figment::{Figment, providers::{Format, Toml, Env}};
use figment::value::{Map, Dict, magic::RelativePathBuf};

#[derive(Deserialize)]
struct SmtpServerConfig {
    smtp_server: String,
    smtp_port: u16,
    smtp_username: String,
    smtp_password: String,
    from_address: String,
    tpl_path: RelativePathBuf
}

#[derive(Deserialize)]
pub struct Config {
    smtp_config: SmtpServerConfig,

}

impl Default for Config {
    fn default() -> Config {

    }
}
impl Config {
    fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    pub fn figment() -> Figment {
        Figment::from(Config::default())
            .merge(Toml::file(Env::var_or("AUTHNOSQL_CONFIG", "AuthNoSql.toml").nested()))
            .merge(Env::prefixed("AUTHNOSQL_"))
    }
}

impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Library Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error>  {
        figment::providers::Serialized::defaults(Config::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        // Optionally, a profile that's selected by default.
    }
}

impl Config {
    pub const PORT: &'static str = "smtp_port";
}