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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn gamestore_test() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db = sled::open(temp_dir.path())?;
        let tree = db.open_tree("games")?;
        let store = GameStore::new(tree);

        // Create test game
        let game = Game {
            id: Uuid::new_v4().to_string(),
            team_name: "Test Team".into(),
            start_time: Some(Utc::now()),
            end_time: None,
            score: 42,
        };

        // Insert
        store.insert(&game).expect("insert game");

        // Retrieve
        let fetched = store.get(&game.id).expect("get game").expect("game not found");

        // Assert
        assert_eq!(fetched.id, game.id);
        assert_eq!(fetched.team_name, "Test Team");
        assert_eq!(fetched.score, 42);

        Ok(())
    }
}