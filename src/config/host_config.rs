use anyhow::{Context, bail};
use serde::Deserialize;
use std::env;

/// A single Deluge daemon endpoint to manage.
#[derive(Debug, Clone, Deserialize)]
pub struct HostConfig {
    /// Daemon hostname or IP address.
    pub host: String,

    /// Daemon TCP port. Defaults to `58846`.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Username for daemon authentication. Defaults to `localclient`.
    #[serde(default = "default_username")]
    pub username: String,

    /// Plaintext password. Mutually exclusive with [`HostConfig::password_env`].
    #[serde(default)]
    pub password: Option<String>,

    /// Name of the environment variable holding the password. Mutually
    /// exclusive with [`HostConfig::password`].
    #[serde(default)]
    pub password_env: Option<String>,
}

impl HostConfig {
    /// Resolve the password for this host.
    ///
    /// Returns the plaintext [`HostConfig::password`] when set, or reads the
    /// value of the environment variable named by [`HostConfig::password_env`].
    ///
    /// # Errors
    ///
    /// Returns an error if neither field is set, both are set, or the named
    /// environment variable is missing or empty.
    pub fn resolve_password(&self) -> anyhow::Result<String> {
        match (&self.password, &self.password_env) {
            (Some(pw), None) => Ok(pw.clone()),
            (None, Some(var)) => {
                let value = env::var(var)
                    .with_context(|| format!("password_env variable {var} is not set"))?;
                if value.is_empty() {
                    bail!("password_env variable {var} is set but empty");
                }
                Ok(value)
            }
            _ => bail!("password or password_env must be set"),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.host.is_empty() {
            bail!("host must not be empty");
        }
        if self.port == 0 {
            bail!("port must be > 0, got 0");
        }
        if self.username.is_empty() {
            bail!("username must not be empty");
        }
        match (&self.password, &self.password_env) {
            (Some(_), Some(_)) => {
                bail!("only one of password or password_env may be set");
            }
            (None, None) => {
                bail!("password or password_env must be set");
            }
            _ => Ok(()),
        }
    }
}

fn default_port() -> u16 {
    58846
}

fn default_username() -> String {
    String::from("localclient")
}
