use clap::Args;

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
}

impl CliConfig {
    pub fn resolve(&self) -> anyhow::Result<ResolvedConfig> {
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

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub record: Option<String>,
}
