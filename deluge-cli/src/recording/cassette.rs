use crate::recording::Interaction;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cassette {
    pub version: u32,
    pub recorded_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daemon_version: Option<String>,
    pub interactions: Vec<Interaction>,
}

impl Cassette {
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow::anyhow!("failed to serialize cassette: {e}"))?;

        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, json)
            .map_err(|e| anyhow::anyhow!("failed to write temporary cassette: {e}"))?;

        fs::rename(&temp_path, path)
            .map_err(|e| anyhow::anyhow!("failed to rename temporary cassette: {e}"))?;

        Ok(())
    }

    pub fn load(path: &Path) -> anyhow::Result<Cassette> {
        let data = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("failed to read cassette: {e}"))?;
        serde_json::from_str(&data).map_err(|e| anyhow::anyhow!("failed to parse cassette: {e}"))
    }
}
