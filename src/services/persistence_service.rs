use crate::persistence::game::{GameStore, GamePersistenceError};
use crate::persistence::game_event::{GameEventStore, GameEvent, GameEventError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceServiceError {
    #[error("Game with ID `{0}` not found")]
    GameNotFound(String),

    #[error("Game error: {0}")]
    Game(#[from] GamePersistenceError),

    #[error("Game event error: {0}")]
    GameEvent(#[from] GameEventError),
}

pub struct PersistenceService<'a> {
    pub game_store: &'a GameStore,
    pub event_store: &'a GameEventStore,
}

impl<'a> PersistenceService<'a> {
    pub fn new(game_store: &'a GameStore, event_store: &'a GameEventStore) -> Self {
        Self {
            game_store,
            event_store,
        }
    }

    pub fn insert_event_if_game_exists(
        &self,
        key: &str,
        event: &GameEvent,
    ) -> Result<(), PersistenceServiceError> {
        let exists = self
            .game_store
            .get(&event.game_id)?
            .is_some();

        if !exists {
            return Err(PersistenceServiceError::GameNotFound(event.game_id.clone()));
        }

        self.event_store.insert(key, event)?;
        Ok(())
    }

    // You can later add:
    // - fn start_new_game(...)
    // - fn complete_game(...)
    // - fn get_all_events_for_game(...)
}