use crate::config::host_config::HostConfig;
use crate::config::rules::Rules;
use anyhow::{Context, bail};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::Duration;

/// Top-level configuration loaded from the TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Interval between retention sweeps.
    #[serde(with = "humantime_serde")]
    pub poll_interval: Duration,

    /// Deluge hosts to manage. Must contain at least one entry.
    #[serde(default)]
    pub hosts: Vec<HostConfig>,

    /// Retention rules applied to every host.
    #[serde(default)]
    pub rules: Rules,
}

impl Config {
    /// Load and validate a configuration from the TOML file at `path`.
    pub fn load(path: &str) -> anyhow::Result<Config> {
        let contents = fs::read_to_string(Path::new(path))
            .with_context(|| format!("failed to read config file {path}"))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("failed to parse config file {path}"))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.poll_interval < Duration::from_secs(1) {
            bail!(
                "poll_interval must be at least 1 second, got {}s",
                self.poll_interval.as_secs()
            );
        }

        if self.hosts.is_empty() {
            bail!("at least one host required");
        }

        for (idx, host) in self.hosts.iter().enumerate() {
            host.validate()
                .with_context(|| format!("host[{idx}] ({}:{})", host.host, host.port))?;
        }

        self.rules.validate()?;
        Ok(())
    }
}
