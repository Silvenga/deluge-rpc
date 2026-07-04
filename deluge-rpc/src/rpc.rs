use crate::models::TorrentInfo;
use async_trait::async_trait;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait DelugeRpc: Send + Sync {
    async fn get_free_space(&self) -> anyhow::Result<u64>;
    async fn get_torrents(&self) -> anyhow::Result<Vec<TorrentInfo>>;
    async fn remove_torrent(&self, id: &str) -> anyhow::Result<bool>;
}
