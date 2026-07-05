use clap::Parser;

/// Automatic torrent retention tool for Deluge.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the TOML config file.
    #[arg(long, default_value = "deluge-retain.toml")]
    pub config: String,

    /// Preview what would be deleted without actually removing torrents.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Enable verbose (TRACE-level) logging.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,

    /// Run a single check cycle and exit (default: watch mode).
    #[arg(long, default_value_t = false)]
    pub once: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn when_dry_run_and_once_flags_then_both_true() {
        let cli = Cli::try_parse_from(["deluge-retain", "--dry-run", "--once"])
            .expect("valid flags must parse");

        assert!(cli.dry_run);
        assert!(cli.once);
        assert!(!cli.verbose);
        assert_eq!(cli.config, "deluge-retain.toml");
    }

    #[test]
    fn when_verbose_flag_then_verbose_true() {
        let cli =
            Cli::try_parse_from(["deluge-retain", "--verbose"]).expect("valid flags must parse");

        assert!(cli.verbose);
        assert!(!cli.dry_run);
        assert!(!cli.once);
    }

    #[test]
    fn when_config_path_provided_then_config_set() {
        let cli = Cli::try_parse_from(["deluge-retain", "--config", "/path/to/config.toml"])
            .expect("valid flags must parse");

        assert_eq!(cli.config, "/path/to/config.toml");
    }

    #[test]
    fn when_no_flags_then_all_defaults() {
        let cli = Cli::try_parse_from(["deluge-retain"]).expect("valid flags must parse");

        assert!(!cli.dry_run);
        assert!(!cli.verbose);
        assert!(!cli.once);
        assert_eq!(cli.config, "deluge-retain.toml");
    }

    #[test]
    fn when_unknown_flag_then_parse_error() {
        let result = Cli::try_parse_from(["deluge-retain", "--unknown"]);

        assert_matches!(result, Err(_));
    }
}
