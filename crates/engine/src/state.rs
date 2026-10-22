use rustic_core::api::{RepeatMode, Track};
use rustic_core::queue::QueueManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineState {
    #[serde(default)]
    pub queue: Vec<Track>,
    #[serde(default)]
    pub current_index: Option<usize>,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub shuffle: bool,
    #[serde(default)]
    pub repeat: RepeatMode,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default)]
    pub queue_manager: Option<QueueManager>,
}

fn default_volume() -> f32 {
    1.0
}

fn default_seed() -> u64 {
    rand::random()
}

impl Default for EngineState {
    fn default() -> Self {
        Self {
            queue: Vec::new(),
            current_index: None,
            volume: 1.0,
            shuffle: false,
            repeat: RepeatMode::Off,
            seed: rand::random(),
            queue_manager: None,
        }
    }
}

impl EngineState {
    pub fn load(conn: &rusqlite::Connection) -> Self {
        let result: Result<String, _> = conn.query_row(
            "SELECT value FROM app_state WHERE key = 'engine_state'",
            [],
            |row| row.get::<_, String>(0),
        );

        match result {
            Ok(json_str) => serde_json::from_str(&json_str).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, conn: &rusqlite::Connection) {
        if let Ok(json_str) = serde_json::to_string(self) {
            let _ = conn.execute(
                "INSERT INTO app_state (key, value) VALUES ('engine_state', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                [json_str],
            );
        }
    }

    /// Retrieves or initializes a QueueManager from state.
    pub fn to_queue_manager(&self) -> QueueManager {
        let mut qm = if let Some(ref qm) = self.queue_manager {
            qm.clone()
        } else if !self.queue.is_empty() {
            let mut q = QueueManager::new();
            q.actual_queue = self.queue.clone();
            q.cursor = self.current_index.unwrap_or(0).min(self.queue.len().saturating_sub(1));
            q.is_shuffled = self.shuffle;
            q
        } else {
            QueueManager::new()
        };
        qm.autoplay_enabled = true;
        qm
    }
}
