use std::fs;

use clap::Args;
use serde::Deserialize;

#[derive(Args, Debug, Clone)]
pub struct CliConfig {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(long, default_value_t = 58846)]
    pub port: u16,

    #[arg(long, default_value = "localclient")]
    pub user: String,

    #[arg(long, env = "DELUGE_PASSWORD")]
    pub pass: Option<String>,

    #[arg(long)]
    pub record: Option<String>,

    #[arg(long)]
    pub config: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TomlConfig {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl CliConfig {
    pub fn resolve(&self) -> anyhow::Result<ResolvedConfig> {
        if let Some(config_path) = &self.config {
            let contents = fs::read_to_string(config_path)
                .map_err(|e| anyhow::anyhow!("failed to read config file {config_path}: {e}"))?;
            let toml_config: TomlConfig = toml::from_str(&contents)
                .map_err(|e| anyhow::anyhow!("failed to parse config file {config_path}: {e}"))?;

            let host = host_or(&self.host, toml_config.host.as_deref(), "127.0.0.1");
            let port = port_or(self.port, toml_config.port, 58846);
            let user = user_or(&self.user, toml_config.user.as_deref(), "localclient");
            let pass = toml_config.password.or(self.pass.clone()).ok_or_else(|| {
                anyhow::anyhow!("password required: use --pass flag or set DELUGE_PASSWORD env var or configure in TOML config")
            })?;

            Ok(ResolvedConfig {
                host: host.to_owned(),
                port,
                user: user.to_owned(),
                pass,
                record: self.record.clone(),
            })
        } else {
            let pass = self.pass.clone().ok_or_else(|| {
                anyhow::anyhow!("password required: use --pass flag or set DELUGE_PASSWORD env var")
            })?;

            Ok(ResolvedConfig {
                host: self.host.clone(),
                port: self.port,
                user: self.user.clone(),
                pass,
                record: self.record.clone(),
            })
        }
    }
}

fn host_or<'a>(cli_val: &'a str, toml_val: Option<&'a str>, default: &'a str) -> &'a str {
    if cli_val != "127.0.0.1" {
        cli_val
    } else {
        toml_val.unwrap_or(default)
    }
}

fn port_or(cli_val: u16, toml_val: Option<u16>, default: u16) -> u16 {
    if cli_val != 58846 {
        cli_val
    } else {
        toml_val.unwrap_or(default)
    }
}

fn user_or<'a>(cli_val: &'a str, toml_val: Option<&'a str>, default: &'a str) -> &'a str {
    if cli_val != "localclient" {
        cli_val
    } else {
        toml_val.unwrap_or(default)
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub record: Option<String>,
}
