use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    routing::{get, post},
    Json, Router,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::Infallible, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info, warn};

use crate::{
    agent::{AgentRequest, AgentRunner},
    config::ServerConfig,
    error::{AppError, AppResult},
    session::SessionStore,
};

/// Shared application state
#[derive(Clone)]
struct AppState {
    config: Arc<ServerConfig>,
    session_store: Arc<SessionStore>,
    agent_runner: Arc<AgentRunner>,
    // Track running processes for optional termination
    running_processes: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

/// Spawn request payload
#[derive(Debug, Deserialize)]
struct SpawnRequest {
    agent_type: String,
    prompt: String,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    tools_allowed: Vec<String>,
    system_append: Option<String>,
    resume_id: Option<String>,
    /// Whether to create a new session for resumption
    #[serde(default)]
    create_session: bool,
}

/// Message request payload (for resuming sessions)
#[derive(Debug, Deserialize)]
struct MessageRequest {
    prompt: String,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    tools_allowed: Vec<String>,
    system_append: Option<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

/// Health check endpoint
async fn health() -> Json<HealthResponse> {
    debug!("Health check requested");
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Spawn endpoint - creates a new agent process and streams JSONL output via SSE
async fn spawn(
    State(state): State<AppState>,
    Json(payload): Json<SpawnRequest>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    info!(
        "🚀 Spawn request - agent_type: '{}', create_session: {}, tools: {:?}, prompt_length: {} chars",
        payload.agent_type,
        payload.create_session,
        payload.tools_allowed,
        payload.prompt.len()
    );
    debug!(
        "Spawn request details - flags: {:?}, system_append: {:?}, resume_id: {:?}",
        payload.flags,
        payload
            .system_append
            .as_ref()
            .map(|s| format!("{}...", &s.chars().take(50).collect::<String>())),
        payload.resume_id
    );

    // Create session if requested
    let session_id = if payload.create_session {
        let metadata = state
            .session_store
            .create_session(payload.agent_type.clone())
            .await?;
        info!("📝 Created session: {}", metadata.session_id);
        Some(metadata.session_id)
    } else {
        debug!("No session requested (stateless mode)");
        None
    };

    // Build agent request
    let agent_request = AgentRequest {
        agent_type: payload.agent_type,
        prompt: payload.prompt,
        flags: payload.flags,
        tools_allowed: payload.tools_allowed,
        system_append: payload.system_append,
        resume_id: payload.resume_id.or_else(|| session_id.clone()),
    };

    // Spawn the claude process
    info!("⚡ Spawning Claude CLI process...");
    let (mut child, mut rx) = state.agent_runner.spawn(agent_request).await?;
    info!("✓ Claude process spawned successfully");

    // Get the child PID for monitoring
    let child_pid = child.id();

    // Spawn a task to monitor process exit status
    let monitor_pid = child_pid;
    let monitor_session = session_id.clone();
    let monitor_running_procs = state.running_processes.clone();
    tokio::spawn(async move {
        match child.wait().await {
            Ok(status) => {
                if status.success() {
                    info!(
                        "✅ Process {:?} exited successfully with status: {}",
                        monitor_pid, status
                    );
                } else {
                    warn!(
                        "⚠️  Process {:?} exited with non-zero status: {}",
                        monitor_pid, status
                    );
                }
            }
            Err(e) => {
                tracing::error!("❌ Failed to wait for process {:?}: {}", monitor_pid, e);
            }
        }

        // Clean up from running processes if applicable
        if let Some(sid) = monitor_session {
            monitor_running_procs.lock().await.remove(&sid);
        }
    });

    // Create SSE stream
    let stream_session_id = session_id.clone();
    let running_procs = state.running_processes.clone();
    let stream = async_stream::stream! {
        // Send initial session info if available
        if let Some(ref sid) = stream_session_id {
            info!("📤 Sending session_created event for: {}", sid);
            let event = Event::default()
                .json_data(serde_json::json!({
                    "type": "session_created",
                    "session_id": sid
                }))
                .unwrap();
            yield Ok(event);
        }

        let mut output_count = 0;
        // Stream JSONL lines from claude
        while let Some(result) = rx.recv().await {
            match result {
                Ok(line) => {
                    output_count += 1;
                    if output_count == 1 {
                        info!("📥 First output received from Claude");
                    }
                    debug!("Output line {}: {} bytes", output_count, line.len());
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "output",
                            "data": line
                        }))
                        .unwrap();
                    yield Ok(event);
                }
                Err(e) => {
                    error!("❌ Error from Claude process: {}", e);
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "error",
                            "error": e.to_string()
                        }))
                        .unwrap();
                    yield Ok(event);
                    break;
                }
            }
        }

        if output_count == 0 {
            warn!("⚠️  WARNING: Claude process completed with ZERO output lines!");
            warn!("   This likely indicates a process spawning or execution failure.");
            warn!("   Check that Claude CLI is properly installed and accessible.");
            warn!("   On Windows, ensure .cmd files are executed through cmd.exe wrapper.");
        } else {
            info!("✅ Claude process completed - {} output lines sent", output_count);
        }

        // Send completion event
        let event = Event::default()
            .json_data(serde_json::json!({
                "type": "completed"
            }))
            .unwrap();
        yield Ok(event);

        // Clean up running process
        if let Some(ref sid) = stream_session_id {
            running_procs.lock().await.remove(sid);
            debug!("🧹 Cleaned up process for session: {}", sid);
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Message endpoint - send a message to an existing session
async fn message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(payload): Json<MessageRequest>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    info!(
        "💬 Message request - session_id: {}, tools: {:?}, prompt_length: {} chars",
        session_id,
        payload.tools_allowed,
        payload.prompt.len()
    );

    // Verify session exists and update last_used
    state.session_store.touch_session(&session_id).await?;
    info!("✓ Session found and updated: {}", session_id);

    // Build agent request with resume
    let agent_request = AgentRequest {
        agent_type: "resumed".to_string(),
        prompt: payload.prompt,
        flags: payload.flags,
        tools_allowed: payload.tools_allowed,
        system_append: payload.system_append,
        resume_id: Some(session_id.clone()),
    };

    // Spawn the claude process
    info!("⚡ Resuming Claude session...");
    let (child, mut rx) = state.agent_runner.spawn(agent_request).await?;
    info!("✓ Claude process resumed successfully");

    // Store the process
    state
        .running_processes
        .lock()
        .await
        .insert(session_id.clone(), child);
    debug!("Stored resumed process for session: {}", session_id);

    // Create SSE stream
    let stream_session_id = session_id.clone();
    let running_procs = state.running_processes.clone();
    let stream = async_stream::stream! {
        let mut output_count = 0;
        // Stream JSONL lines from claude
        while let Some(result) = rx.recv().await {
            match result {
                Ok(line) => {
                    output_count += 1;
                    if output_count == 1 {
                        info!("📥 First output received from resumed session");
                    }
                    debug!("Resume output line {}: {} bytes", output_count, line.len());
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "output",
                            "data": line
                        }))
                        .unwrap();
                    yield Ok(event);
                }
                Err(e) => {
                    error!("❌ Error from resumed session: {}", e);
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "error",
                            "error": e.to_string()
                        }))
                        .unwrap();
                    yield Ok(event);
                    break;
                }
            }
        }

        info!("✅ Resumed session completed - {} output lines sent", output_count);
        // Send completion event
        let event = Event::default()
            .json_data(serde_json::json!({
                "type": "completed"
            }))
            .unwrap();
        yield Ok(event);

        // Clean up running process
        running_procs.lock().await.remove(&stream_session_id);
        debug!("🧹 Cleaned up resumed session: {}", stream_session_id);
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Terminate endpoint - forcefully terminate a running agent process
async fn terminate(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("🛑 Terminate request - session_id: {}", session_id);

    let mut processes = state.running_processes.lock().await;

    if let Some(child) = processes.remove(&session_id) {
        AgentRunner::terminate(child).await?;
        info!("✓ Process terminated successfully: {}", session_id);
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Process terminated successfully"
            })),
        ))
    } else {
        warn!("⚠️  No running process found for session: {}", session_id);
        Err(AppError::SessionNotFound(session_id))
    }
}

/// List sessions endpoint
async fn list_sessions(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    info!("📋 List sessions request");
    let session_dir = &state.config.session_dir;
    let mut sessions = Vec::new();

    let mut entries = tokio::fs::read_dir(session_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.ends_with(".json") {
                let session_id = filename.trim_end_matches(".json");
                if let Ok(metadata) = state.session_store.load_session(session_id).await {
                    sessions.push(metadata);
                }
            }
        }
    }

    info!("✓ Found {} sessions", sessions.len());
    debug!("Session details: {:?}", sessions);
    Ok(Json(serde_json::json!({ "sessions": sessions })))
}

/// Build the router with all endpoints
fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/spawn", post(spawn))
        .route("/message/:session_id", post(message))
        .route("/terminate/:session_id", post(terminate))
        .route("/sessions", get(list_sessions))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// Start the HTTP server
pub async fn serve(addr: &str, config: Arc<ServerConfig>) -> anyhow::Result<()> {
    let session_store = Arc::new(SessionStore::new(&config.session_dir));
    let agent_runner = Arc::new(AgentRunner::new(config.claude_path.clone()));
    let running_processes = Arc::new(Mutex::new(HashMap::new()));

    let state = AppState {
        config,
        session_store,
        agent_runner,
        running_processes,
    };

    let app = app(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("🚀 Server listening on http://{}", addr);
    info!("📍 Endpoints: /health, /spawn, /message/:id, /terminate/:id, /sessions");

    axum::serve(listener, app).await?;

    Ok(())
}
