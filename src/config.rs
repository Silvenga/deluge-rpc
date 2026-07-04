//! Configuration parsing and representation.
//!
//! The configuration is read from a TOML file. See the [`Config`] struct for
//! the top-level shape, and the [`Rules`] / [`HostConfig`] structs for the
//! nested sections.

use std::env;
use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context};
use bytesize::ByteSize;
use serde::Deserialize;

/// Top-level configuration loaded from the TOML file.
///
/// Contains the poll interval, the list of Deluge hosts to manage, and the
/// retention [`Rules`].
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Interval between retention sweeps.
    ///
    /// Parsed from a human-readable duration string (e.g. `"5m"`, `"30s"`)
    /// via `humantime_serde`. Stored as a [`std::time::Duration`].
    #[serde(with = "humantime_serde")]
    pub poll_interval: Duration,

    /// Deluge hosts to manage. Must contain at least one entry.
    #[serde(default)]
    pub hosts: Vec<HostConfig>,

    /// Retention rules applied to every host.
    #[serde(default)]
    pub rules: Rules,
}

/// A single Deluge daemon endpoint to manage.
#[derive(Debug, Clone, Deserialize)]
pub struct HostConfig {
    /// Daemon hostname or IP address.
    pub host: String,

    /// Daemon TCP port. Defaults to `58846`.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Username for daemon authentication. Defaults to `localclient`.
    #[expect(dead_code, reason = "consumed by daemon RPC client in task 3")]
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

fn default_port() -> u16 {
    58846
}

fn default_username() -> String {
    String::from("localclient")
}

/// Retention rules shared by all hosts.
#[derive(Debug, Clone, Deserialize)]
pub struct Rules {
    /// Minimum swarm-wide seed count (Deluge `total_seeds`) below which a
    /// torrent is retained. Must be `>= 1`.
    #[serde(default = "default_min_seeders")]
    pub min_seeders: u32,

    /// Minimum age in days a torrent must reach before it is eligible for
    /// deletion. Must be `>= 1`.
    #[serde(default = "default_min_age_days")]
    pub min_age_days: u64,

    /// Free-space threshold below which retention is triggered.
    pub low_water_mark: ByteSize,

    /// Free-space threshold above which retention pauses. Must be greater than
    /// [`Rules::low_water_mark`].
    pub high_water_mark: ByteSize,

    /// Minimum seconds between two deletion operations for the same host.
    /// `0` disables throttling.
    #[serde(default)]
    pub delete_throttle_secs: u64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            min_seeders: default_min_seeders(),
            min_age_days: default_min_age_days(),
            low_water_mark: ByteSize::b(0),
            high_water_mark: ByteSize::b(0),
            delete_throttle_secs: 0,
        }
    }
}

fn default_min_seeders() -> u32 {
    1
}

fn default_min_age_days() -> u64 {
    1
}

impl Config {
    /// Load and validate a configuration from the TOML file at `path`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, the TOML is malformed, or
    /// any validation rule is violated.
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

    fn validate(&self) -> anyhow::Result<()> {
        if self.host.is_empty() {
            bail!("host must not be empty");
        }
        if self.port == 0 {
            bail!("port must be > 0, got 0");
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

impl Rules {
    fn validate(&self) -> anyhow::Result<()> {
        if self.min_seeders < 1 {
            bail!("min_seeders must be >= 1, got {}", self.min_seeders);
        }
        if self.min_age_days < 1 {
            bail!("min_age_days must be >= 1, got {}", self.min_age_days);
        }
        if self.low_water_mark >= self.high_water_mark {
            bail!(
                "low_water_mark must be less than high_water_mark, got low={} high={}",
                self.low_water_mark,
                self.high_water_mark
            );
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests panic on unexpected shapes — that is the test failing"
)]
mod tests {
    use super::*;
    use assert_fs::fixture::FileWriteBin;
    use assert_fs::NamedTempFile;

    fn write_config(contents: &str) -> NamedTempFile {
        let file = NamedTempFile::new(".toml").expect("create tempfile");
        file.write_binary(contents.as_bytes())
            .expect("write tempfile");
        file
    }

    fn err_chain(err: &anyhow::Error) -> String {
        use std::fmt::Write;
        let mut out = String::new();
        for cause in err.chain() {
            let _ = write!(out, "{cause}: ");
        }
        out.trim_end_matches(": ").to_owned()
    }

    const VALID_CONFIG: &str = r#"
poll_interval = "5m"

[[hosts]]
host = "deluge-1"
port = 58846
username = "localclient"
password = "secret-1"

[[hosts]]
host = "deluge-2"
port = 58846
username = "localclient"
password_env = "DELUGE_PASSWORD_2"

[rules]
min_seeders = 3
min_age_days = 7
low_water_mark = "50 GiB"
high_water_mark = "100 GiB"
delete_throttle_secs = 30
"#;

    fn valid_config() -> Config {
        let file = write_config(VALID_CONFIG);
        Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse")
    }

    #[test]
    fn when_config_valid_then_parses_correctly() {
        let config = valid_config();
        assert_eq!(config.poll_interval, Duration::from_secs(300));
        assert_eq!(config.hosts.len(), 2);
        assert_eq!(config.hosts[0].host, "deluge-1");
        assert_eq!(config.hosts[0].port, 58846);
        assert_eq!(config.hosts[0].username, "localclient");
        assert_eq!(config.hosts[0].password.as_deref(), Some("secret-1"));
        assert_eq!(
            config.hosts[1].password_env.as_deref(),
            Some("DELUGE_PASSWORD_2")
        );
        assert_eq!(config.rules.min_seeders, 3);
        assert_eq!(config.rules.min_age_days, 7);
        assert_eq!(config.rules.low_water_mark, ByteSize::gib(50));
        assert_eq!(config.rules.high_water_mark, ByteSize::gib(100));
        assert_eq!(config.rules.delete_throttle_secs, 30);
    }

    #[test]
    fn when_port_not_set_then_defaults_to_58846() {
        let contents = r#"
poll_interval = "5m"

[[hosts]]
host = "deluge-1"
username = "localclient"
password = "secret-1"

[rules]
min_seeders = 3
min_age_days = 7
low_water_mark = "50 GiB"
high_water_mark = "100 GiB"
"#;
        let file = write_config(contents);
        let config =
            Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse");
        assert_eq!(config.hosts[0].port, 58846);
    }

    #[test]
    fn when_username_not_set_then_defaults_to_localclient() {
        let contents = r#"
poll_interval = "5m"

[[hosts]]
host = "deluge-1"
port = 58846
password = "secret-1"

[rules]
min_seeders = 3
min_age_days = 7
low_water_mark = "50 GiB"
high_water_mark = "100 GiB"
"#;
        let file = write_config(contents);
        let config =
            Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse");
        assert_eq!(config.hosts[0].username, "localclient");
    }

    #[test]
    fn when_host_empty_then_error() {
        let contents = VALID_CONFIG.replacen(r#"host = "deluge-1""#, r#"host = """#, 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("empty host should error");
        assert!(
            err_chain(&err).contains("host must not be empty"),
            "got: {err}"
        );
    }

    #[test]
    fn when_port_zero_then_error() {
        let contents = VALID_CONFIG.replacen("port = 58846", "port = 0", 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("zero port should error");
        assert!(
            err_chain(&err).contains("port must be > 0, got 0"),
            "got: {err}"
        );
    }

    #[test]
    fn when_password_env_set_then_reads_from_env() {
        // SAFETY: unique env var name avoids races with other tests; the
        // var is removed before this test returns.
        unsafe { env::set_var("DR_TEST_PW_SET_OK", "env-secret") };
        let contents = VALID_CONFIG.replacen("DELUGE_PASSWORD_2", "DR_TEST_PW_SET_OK", 1);
        let file = write_config(&contents);
        let config =
            Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse");
        let pw = config.hosts[1]
            .resolve_password()
            .expect("resolve env password");
        assert_eq!(pw, "env-secret");
        // SAFETY: see above.
        unsafe { env::remove_var("DR_TEST_PW_SET_OK") };
    }

    #[test]
    fn when_password_plaintext_then_resolve_returns_it() {
        let config = valid_config();
        let pw = config.hosts[0]
            .resolve_password()
            .expect("resolve plaintext");
        assert_eq!(pw, "secret-1");
    }

    #[test]
    fn when_low_water_mark_not_less_than_high_then_error() {
        let contents = VALID_CONFIG.replacen(
            r#"low_water_mark = "50 GiB""#,
            r#"low_water_mark = "100 GiB""#,
            1,
        );
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("equal water marks should error");
        assert!(
            err_chain(&err).contains("low_water_mark must be less than high_water_mark"),
            "got: {err}"
        );
    }

    #[test]
    fn when_poll_interval_below_one_second_then_error() {
        let contents =
            VALID_CONFIG.replacen(r#"poll_interval = "5m""#, r#"poll_interval = "0s""#, 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("zero poll interval should error");
        assert!(
            err_chain(&err).contains("poll_interval must be at least 1 second"),
            "got: {err}"
        );
    }

    #[test]
    fn when_hosts_empty_then_error() {
        let contents = r#"
poll_interval = "5m"
hosts = []
[rules]
min_seeders = 3
min_age_days = 7
low_water_mark = "50 GiB"
high_water_mark = "100 GiB"
"#;
        let file = write_config(contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("empty hosts should error");
        assert!(
            err_chain(&err).contains("at least one host required"),
            "got: {err}"
        );
    }

    #[test]
    fn when_both_password_fields_set_then_error() {
        let contents = VALID_CONFIG.replacen(
            r#"password = "secret-1""#,
            r#"password = "secret-1"
password_env = "DELUGE_PASSWORD_1""#,
            1,
        );
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("both password fields should error");
        assert!(
            err_chain(&err).contains("only one of password or password_env may be set"),
            "got: {err}"
        );
    }

    #[test]
    fn when_neither_password_field_set_then_error() {
        let contents = VALID_CONFIG.replacen(r#"password = "secret-1""#, "", 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("no password field should error");
        assert!(
            err_chain(&err).contains("password or password_env must be set"),
            "got: {err}"
        );
    }

    #[test]
    fn when_min_seeders_zero_then_error() {
        let contents = VALID_CONFIG.replacen("min_seeders = 3", "min_seeders = 0", 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("zero min_seeders should error");
        assert!(
            err_chain(&err).contains("min_seeders must be >= 1"),
            "got: {err}"
        );
    }

    #[test]
    fn when_min_age_days_zero_then_error() {
        let contents = VALID_CONFIG.replacen("min_age_days = 7", "min_age_days = 0", 1);
        let file = write_config(&contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("zero min_age_days should error");
        assert!(
            err_chain(&err).contains("min_age_days must be >= 1"),
            "got: {err}"
        );
    }

    #[test]
    fn when_missing_rules_section_then_error() {
        let contents = r#"
poll_interval = "5m"
[[hosts]]
host = "deluge"
port = 58846
username = "localclient"
password = "secret"
"#;
        let file = write_config(contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("missing rules should error");
        assert!(
            err_chain(&err).contains("low_water_mark must be less than high_water_mark"),
            "default rules (both zero) should fail validation: {err}"
        );
    }

    #[test]
    fn when_missing_poll_interval_then_error() {
        let contents = r#"
[[hosts]]
host = "deluge"
port = 58846
username = "localclient"
password = "secret"
[rules]
min_seeders = 3
min_age_days = 7
low_water_mark = "50 GiB"
high_water_mark = "100 GiB"
"#;
        let file = write_config(contents);
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("missing poll_interval should error");
        assert!(err_chain(&err).contains("poll_interval"), "got: {err}");
    }

    #[test]
    fn when_password_env_missing_var_then_resolve_errors() {
        // SAFETY: unique env var name avoids races with other tests.
        unsafe { env::remove_var("DR_TEST_PW_MISSING") };
        let contents = VALID_CONFIG.replacen("DELUGE_PASSWORD_2", "DR_TEST_PW_MISSING", 1);
        let file = write_config(&contents);
        let config =
            Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse");
        let err = config.hosts[1]
            .resolve_password()
            .expect_err("missing env var should error");
        assert!(err_chain(&err).contains("DR_TEST_PW_MISSING"), "got: {err}");
    }

    #[test]
    fn when_password_env_empty_var_then_resolve_errors() {
        // SAFETY: unique env var name avoids races with other tests; removed
        // before this test returns.
        unsafe { env::set_var("DR_TEST_PW_EMPTY", "") };
        let contents = VALID_CONFIG.replacen("DELUGE_PASSWORD_2", "DR_TEST_PW_EMPTY", 1);
        let file = write_config(&contents);
        let config =
            Config::load(file.to_str().expect("path is utf-8")).expect("valid config should parse");
        let err = config.hosts[1]
            .resolve_password()
            .expect_err("empty env var should error");
        assert!(err_chain(&err).contains("set but empty"), "got: {err}");
        // SAFETY: see above.
        unsafe { env::remove_var("DR_TEST_PW_EMPTY") };
    }

    #[test]
    fn when_file_missing_then_error() {
        let err = Config::load("/nonexistent/path/to/config.toml")
            .expect_err("missing file should error");
        assert!(
            err_chain(&err).contains("failed to read config file"),
            "got: {err}"
        );
    }

    #[test]
    fn when_malformed_toml_then_error() {
        let file = write_config("this is not = valid = toml =");
        let err = Config::load(file.to_str().expect("path is utf-8"))
            .expect_err("malformed toml should error");
        assert!(
            err_chain(&err).contains("failed to parse config file"),
            "got: {err}"
        );
    }

    #[test]
    fn when_resolve_password_on_host_with_neither_then_error() {
        let host = HostConfig {
            host: String::from("deluge"),
            port: 58846,
            username: String::from("localclient"),
            password: None,
            password_env: None,
        };
        let err = host
            .resolve_password()
            .expect_err("neither field should error");
        assert!(
            err_chain(&err).contains("password or password_env must be set"),
            "got: {err}"
        );
    }

    #[test]
    fn when_resolve_password_on_host_with_both_then_error() {
        let host = HostConfig {
            host: String::from("deluge"),
            port: 58846,
            username: String::from("localclient"),
            password: Some(String::from("a")),
            password_env: Some(String::from("B")),
        };
        let err = host
            .resolve_password()
            .expect_err("both fields should error");
        assert!(
            err_chain(&err).contains("password or password_env must be set"),
            "got: {err}"
        );
    }
}
