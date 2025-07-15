mod settings;
mod error;
mod persistence;
mod services;

use crate::settings::Settings;
use error::AppError;
use std::env;

use persistence::game::{Game, GameStore};
use persistence::game_event::{GameEventType, GameEvent, GameEventStore};
// use persistence_service::{PersistenceService, PersistenceServiceError};
use services::persistence_service::PersistenceService;

use chrono::Utc;
use uuid::Uuid;

fn load_and_update_settings() -> Result<Settings, AppError> {
    let path = env::var("SETTINGS_PATH").unwrap_or_else(|_| "settings.json".to_string());

    println!("Using settings file: {}", path);

    let mut settings = Settings::load_from_file(&path)?;
    settings.max_connections += 1;
    settings.save_to_file(&path)?;
    Ok(settings)
}

fn main() -> Result<(), AppError> {
    let settings = load_and_update_settings()?;
    println!("App is running with max_connections: {}", settings.max_connections);

    // Open Sled DB
    let db = sled::open("game_db")?;

    // Initialize trees
    let games_tree = db.open_tree("games")?;
    let events_tree = db.open_tree("game_events")?;

    // Set up stores
    let game_store = GameStore::new(games_tree);
    let event_store = GameEventStore::new(events_tree);

    // Create service
    let service = PersistenceService::new(&game_store, &event_store);

    // ───────────────────────────────
    // Step 1: Create and store a new game
    // ───────────────────────────────
    let game_id = "game-001".to_string();
    let game = Game {
        id: game_id.clone(),
        team_name: "Team Bravo".into(),
        start_time: Some(Utc::now()),
        end_time: None,
        score: 0,
    };

    game_store.insert(&game)?;
    println!("Created game: {:?}", game);

    // ───────────────────────────────
    // Step 2: Log an event for the game
    // ───────────────────────────────
    let event = GameEvent {
        game_id: game_id.clone(),
        event_type: GameEventType::Start,
        timestamp: Utc::now(),
    };

    let event_id = Uuid::new_v4().to_string();
    match service.insert_event_if_game_exists(&event_id, &event) {
        Ok(()) => println!("Logged event: {:?}", event),
        Err(e) => eprintln!("Failed to log event: {e}"),
    }

    // ───────────────────────────────
    // Step 3: Fetch and print game back from DB
    // ───────────────────────────────
    if let Some(fetched) = game_store.get(&game_id)? {
        println!("Fetched game from DB: {:?}", fetched);
    }

    Ok(())
}