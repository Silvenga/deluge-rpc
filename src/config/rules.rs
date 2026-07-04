use anyhow::bail;
use bytesize::ByteSize;
use serde::Deserialize;

/// Retention rules
#[derive(Debug, Clone, Deserialize)]
pub struct Rules {
    /// Minimum swarm-wide seed count (Deluge `total_seeds`) below which a torrent is retained. Must be `>= 1`.
    #[serde(default = "default_min_seeders")]
    pub min_seeders: u32,

    /// Minimum age in days a torrent must reach before it is eligible for deletion. Must be `>= 1`.
    #[serde(default = "default_min_age_days")]
    pub min_age_days: u64,

    /// Free-space threshold below which retention is triggered.
    pub low_water_mark: ByteSize,

    /// Free-space threshold above which retention pauses. Must be greater than [`Rules::low_water_mark`].
    pub high_water_mark: ByteSize,

    /// Minimum seconds between two deletion operations for the same host. `0` disables throttling.
    #[serde(default)]
    pub delete_throttle_secs: u64,
}

impl Rules {
    pub fn validate(&self) -> anyhow::Result<()> {
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
