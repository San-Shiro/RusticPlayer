use async_trait::async_trait;
use rustic_core::api::Command;

#[derive(Debug, thiserror::Error)]
pub enum JamError {
    #[error("connection failed: {0}")]
    Connect(String),
    #[error("transport closed")]
    Closed,
    #[error("protocol error: {0}")]
    Protocol(String),
}

#[async_trait]
pub trait JamTransport: Send + Sync {
    async fn connect(&mut self, endpoint: &str) -> Result<(), JamError>;
    async fn send(&mut self, cmd: &Command) -> Result<(), JamError>;
    async fn recv(&mut self) -> Result<Option<Command>, JamError>;
    async fn disconnect(&mut self) -> Result<(), JamError>;
}
