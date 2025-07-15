use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sled::{Tree};
use bincode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GamePersistenceError {
    #[error("Sled error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub team_name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub score: u32,
}

pub struct GameStore {
    tree: Tree,
}

impl GameStore {
    pub fn new(tree: Tree) -> Self {
        Self { tree }
    }

    pub fn insert(&self, game: &Game) -> Result<(), GamePersistenceError> {
        let bytes = bincode::serialize(game)?;
        self.tree.insert(game.id.as_bytes(), bytes)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<Game>, GamePersistenceError> {
        if let Some(bytes) = self.tree.get(id)? {
            Ok(Some(bincode::deserialize(&bytes)?))
        } else {
            Ok(None)
        }
    }

    pub fn all(&self) -> Result<Vec<Game>, GamePersistenceError> {
        self.tree
            .iter()
            .map(|entry| {
                let (_, val) = entry?;
                Ok(bincode::deserialize(&val)?)
            })
            .collect()
    }
}