use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct TorrentInfo {
    #[serde(skip)]
    pub info_hash: String,
    pub name: String,
    pub state: String,
    pub progress: f64,
    pub ratio: f64,
    pub total_seeds: u32,
    pub num_seeds: u32,
    pub time_added: i64,
    pub total_done: u64,
    pub total_uploaded: u64,
    pub is_finished: bool,
    pub download_location: String,
}
