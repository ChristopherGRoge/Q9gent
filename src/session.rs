use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// Minimal session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub agent_type: String,
    pub created_at: u64,
    pub last_used: u64,
}

/// Session store for persisting minimal metadata
pub struct SessionStore {
    base_dir: PathBuf,
}

impl SessionStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn session_path(&self, session_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", session_id))
    }

    /// Create a new session and persist metadata
    pub async fn create_session(&self, agent_type: String) -> AppResult<SessionMetadata> {
        let session_id = Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let metadata = SessionMetadata {
            session_id: session_id.clone(),
            agent_type,
            created_at: now,
            last_used: now,
        };

        self.save_session(&metadata).await?;
        Ok(metadata)
    }

    /// Save session metadata to disk
    pub async fn save_session(&self, metadata: &SessionMetadata) -> AppResult<()> {
        let path = self.session_path(&metadata.session_id);
        let json = serde_json::to_string_pretty(metadata)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// Load session metadata from disk
    pub async fn load_session(&self, session_id: &str) -> AppResult<SessionMetadata> {
        let path = self.session_path(session_id);
        
        if !path.exists() {
            return Err(AppError::SessionNotFound(session_id.to_string()));
        }

        let json = fs::read_to_string(path).await?;
        let metadata: SessionMetadata = serde_json::from_str(&json)?;
        Ok(metadata)
    }

    /// Update the last_used timestamp
    pub async fn touch_session(&self, session_id: &str) -> AppResult<()> {
        let mut metadata = self.load_session(session_id).await?;
        metadata.last_used = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.save_session(&metadata).await?;
        Ok(())
    }

    /// Delete session metadata
    pub async fn delete_session(&self, session_id: &str) -> AppResult<()> {
        let path = self.session_path(session_id);
        if path.exists() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

