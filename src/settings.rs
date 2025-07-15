/*
Read and write settings to a settings.json.
*/

use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::{Write, ErrorKind};
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Errors
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("Settings file not found")]
    FileNotFound,

    #[error("Permission denied when accessing settings file")]
    PermissionDenied,

    #[error("Unexpected I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Unknown error")]
    Unknown,
}

// ─────────────────────────────────────────────────────────────
// Settings Struct
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub project_name: String,
    pub version: String,
    pub debug: bool,
    pub max_connections: u32,
}

impl Settings {
    pub fn load_from_file(path: &str) -> Result<Self, SettingsError> {
        let content = match read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == ErrorKind::NotFound => return Err(SettingsError::FileNotFound),
            Err(e) if e.kind() == ErrorKind::PermissionDenied => return Err(SettingsError::PermissionDenied),
            Err(e) if e.kind() == ErrorKind::Other => return Err(SettingsError::Io(e)),
            Err(_) => return Err(SettingsError::Unknown),
        };

        let settings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), SettingsError> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_save_and_load_settings_successfully() {
        let settings = Settings {
            project_name: "EscapeRoom".into(),
            version: "1.0.0".into(),
            debug: true,
            max_connections: 42,
        };

        // Write to a temp file
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_str().unwrap();

        settings.save_to_file(path).unwrap();
        let loaded = Settings::load_from_file(path).unwrap();

        assert_eq!(loaded.project_name, "EscapeRoom");
        assert_eq!(loaded.version, "1.0.0");
        assert_eq!(loaded.debug, true);
        assert_eq!(loaded.max_connections, 42);
    }

    #[test]
    fn test_load_missing_file_returns_file_not_found() {
        let result = Settings::load_from_file("does_not_exist.json");
        match result {
            Err(SettingsError::FileNotFound) => {} // pass
            other => panic!("Expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_load_malformed_json_returns_parse_error() {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{{ not: valid: json }}").unwrap(); // malformed JSON
        let path = file.path().to_str().unwrap();

        let result = Settings::load_from_file(path);
        match result {
            Err(SettingsError::Parse(_)) => {} // pass
            other => panic!("Expected Parse error, got {:?}", other),
        }
    }
}