use clap::Parser;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize, Clone)]
#[command(author, version, about, long_about = None)]
#[serde(default)]
pub struct LngAppConfig {
    #[arg(short, long, env = "LNG_BROKER_URL", default_value = "localhost")]
    pub target_host: String,

    #[arg(short, long, env = "LNG_PORT", default_value_t = 1883)]
    pub port: u16,

    #[arg(long, env = "LNG_USE_TLS", default_value_t = false)]
    pub use_tls: bool,

    #[arg(short, long, env = "LNG_USERNAME")]
    pub username: Option<String>,

    #[arg(short, long, env = "LNG_PASSWORD")]
    pub password: Option<String>,

    #[arg(short, long, env = "LNG_CONNECTIONS", default_value_t = 100)]
    pub connections: usize,

    #[arg(short, long, env = "LNG_INTERVAL_MS", default_value_t = 1000)]
    pub interval_ms: u64,

    #[arg(long, env = "LNG_PAYLOAD_TEMPLATE", default_value = "{\"id\": {{id}}, \"temp\": {{random}}}")]
    pub payload_template: String,

    #[arg(long, env = "LNG_RAMP_UP_RATE", default_value_t = 100)]
    pub ramp_up_rate: usize,
}

impl Default for LngAppConfig {
    fn default() -> Self {
        Self {
            target_host: "localhost".to_string(),
            port: 1883,
            use_tls: false,
            username: None,
            password: None,
            connections: 100,
            interval_ms: 1000,
            payload_template: "{\"id\": {{id}}, \"temp\": {{random}}}".to_string(),
            ramp_up_rate: 100,
        }
    }
}

impl LngAppConfig {
    pub fn build() -> Result<Self, ConfigError> {
        // load configuration from toml file and environment variables
        let s = Config::builder()
            .add_source(File::with_name("lng-config").required(false))
            .add_source(Environment::with_prefix("LNG"))
            .build()?;

        // baseline config from file/env; serde(default) ensures all fields exist
        let mut file_cfg: LngAppConfig = s.try_deserialize()?;

        // parse CLI/ENV with clap (env vars are handled by clap attr)
        let cli = LngAppConfig::parse();
        let defaults = LngAppConfig::default();

        // override file values only when CLI/env values differ from their defaults
        if cli.target_host != defaults.target_host {
            file_cfg.target_host = cli.target_host;
        }
        if cli.port != defaults.port {
            file_cfg.port = cli.port;
        }
        if cli.use_tls != defaults.use_tls {
            file_cfg.use_tls = cli.use_tls;
        }
        if cli.username != defaults.username {
            file_cfg.username = cli.username.clone();
        }
        if cli.password != defaults.password {
            file_cfg.password = cli.password.clone();
        }
        if cli.connections != defaults.connections {
            file_cfg.connections = cli.connections;
        }
        if cli.interval_ms != defaults.interval_ms {
            file_cfg.interval_ms = cli.interval_ms;
        }
        if cli.payload_template != defaults.payload_template {
            file_cfg.payload_template = cli.payload_template.clone();
        }
        if cli.ramp_up_rate != defaults.ramp_up_rate {
            file_cfg.ramp_up_rate = cli.ramp_up_rate;
        }

        Ok(file_cfg)
    }
}
