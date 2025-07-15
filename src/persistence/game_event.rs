use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sled::Tree;
use bincode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GameEventError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GameEventType {
    Start,
    Attempt,
    Solve,
    LevelUp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameEvent {
    pub game_id: String,
    pub event_type: GameEventType,
    pub timestamp: DateTime<Utc>,
}

pub struct GameEventStore {
    tree: Tree,
}

impl GameEventStore {
    pub fn new(tree: Tree) -> Self {
        Self { tree }
    }

    pub fn insert(&self, key: &str, event: &GameEvent) -> Result<(), GameEventError> {
        let bytes = bincode::serialize(event)?;
        self.tree.insert(key.as_bytes(), bytes)?;
        Ok(())
    }

    pub fn all(&self) -> Result<Vec<GameEvent>, GameEventError> {
        self.tree
            .iter()
            .map(|entry| {
                let (_, val) = entry?;
                Ok(bincode::deserialize(&val)?)
            })
            .collect()
    }
}