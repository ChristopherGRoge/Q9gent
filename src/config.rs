/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the claude CLI executable
    pub claude_path: String,
    
    /// Directory for session metadata storage
    pub session_dir: String,
}
