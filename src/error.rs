use thiserror::Error;
use crate::settings::SettingsError;
use crate::persistence::game_event::GameEventError;
use crate::services::persistence_service::PersistenceServiceError;
use crate::persistence::game::GamePersistenceError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Settings error: {0}")]
    Settings(#[from] SettingsError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] sled::Error),

    #[error("Game event error: {0}")]
    GameEventStore(#[from] GameEventError),

    #[error("Service error: {0}")]
    Service(#[from] PersistenceServiceError),

    #[error("Game store error: {0}")]
    GameStore(#[from] GamePersistenceError),
}