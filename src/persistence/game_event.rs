use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sled::Tree;
use bincode;
use tempfile::TempDir;
use thiserror::Error;
use uuid::Uuid;
use crate::persistence::game::GameStore;
use crate::services::persistence_service::{PersistenceService, PersistenceServiceError};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::game_event::{GameEventStore, GameEvent, GameEventType};
    use tempfile::TempDir;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_insert_and_all_game_events() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db = sled::open(temp_dir.path())?;
        let tree = db.open_tree("game_events")?;
        let store = GameEventStore::new(tree);

        let event1 = GameEvent {
            game_id: "game-1".to_string(),
            event_type: GameEventType::Start,
            timestamp: Utc::now(),
        };

        let event2 = GameEvent {
            game_id: "game-2".to_string(),
            event_type: GameEventType::Solve,
            timestamp: Utc::now(),
        };

        store.insert(&Uuid::new_v4().to_string(), &event1)?;
        store.insert(&Uuid::new_v4().to_string(), &event2)?;

        let all_events = store.all()?;
        assert_eq!(all_events.len(), 2);


        Ok(())
    }
}
    #[test]
    fn test_insert_event_fails_when_game_missing() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db = sled::open(temp_dir.path())?;
        let game_tree = db.open_tree("games")?;
        let event_tree = db.open_tree("game_events")?;
        let game_store = GameStore::new(game_tree);
        let event_store = GameEventStore::new(event_tree);

        let service = PersistenceService::new(&game_store, &event_store);

        let event = GameEvent {
            game_id: "missing-game".to_string(),
            event_type: GameEventType::Attempt,
            timestamp: Utc::now(),
        };

        let result = service.insert_event_if_game_exists(&Uuid::new_v4().to_string(), &event);

        assert!(matches!(result, Err(PersistenceServiceError::GameNotFound(gid)) if gid == "missing-game"));

        Ok(())
    }