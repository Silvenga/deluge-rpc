//! Tracing and logging initialisation.

use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialise the global tracing subscriber.
///
/// Uses `try_init` internally so repeated calls do not panic — the second call
/// simply returns `Err` which we ignore, leaving the first subscriber in place.
pub fn init_tracing(verbose: bool) {
    let level = if verbose {
        LevelFilter::TRACE
    } else {
        LevelFilter::INFO
    };

    let fmt_layer = layer().with_target(false);

    let _ = tracing_subscriber::registry()
        .with(level)
        .with(fmt_layer)
        .try_init();

    log_panics::init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_init_tracing_with_default_then_does_not_panic() {
        init_tracing(false);
    }

    #[test]
    fn when_init_tracing_with_verbose_then_does_not_panic() {
        init_tracing(true);
    }
}
