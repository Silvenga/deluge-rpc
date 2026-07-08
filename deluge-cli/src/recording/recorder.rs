use crate::recording::{Cassette, Interaction, Request, Response};
use anyhow::Context;
use deluge_rpc_client::{DelugeClientBuilder, RecordedInteraction, RecordedResponse, RencodeValue};
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct Recorder {
    path: String,
    recorder_tx: mpsc::Sender<RecordedInteraction>,
    recorder_rx: mpsc::Receiver<RecordedInteraction>,
}

impl Recorder {
    pub fn new(path: impl Into<String>) -> Self {
        let (recorder_tx, recorder_rx) = mpsc::channel::<RecordedInteraction>(256);
        Self {
            path: path.into(),
            recorder_tx,
            recorder_rx,
        }
    }

    pub fn configure_client(&mut self, builder: DelugeClientBuilder) -> DelugeClientBuilder {
        builder.with_recorder(self.recorder_tx.clone())
    }

    pub async fn persist(&mut self) -> anyhow::Result<()> {
        let mut interactions = Vec::new();
        while let Some(recorded) = self.recorder_rx.recv().await {
            interactions.push(to_interaction(recorded));
        }

        let path = PathBuf::from(self.path.clone());

        let mut existing = Vec::new();
        if path.exists() {
            let cassette = Cassette::load(&path).context("failed to load existing cassette")?;
            existing = cassette.interactions;
        }

        existing.extend(interactions);

        let cassette = Cassette {
            version: 1,
            recorded_at: chrono::Utc::now().to_rfc3339(),
            daemon_version: None,
            interactions: existing,
        };
        cassette
            .save(&path)
            .context("failed to write cassette file")?;
        tracing::info!("cassette written to {}", self.path);

        Ok(())
    }
}

fn to_interaction(recorded: RecordedInteraction) -> Interaction {
    Interaction {
        request: Request {
            method: recorded.request.method,
            args: RencodeValue::List(recorded.request.args),
            kwargs: RencodeValue::Dict(recorded.request.kwargs),
        },
        response: match recorded.response {
            RecordedResponse::Ok { value } => Response::Ok { value },
            RecordedResponse::Error {
                exc_type,
                exc_msg,
                traceback,
            } => Response::Error {
                exc_type,
                exc_msg,
                traceback,
            },
        },
    }
}
