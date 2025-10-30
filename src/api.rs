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
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::Infallible,
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

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

/// Spawn response
#[derive(Debug, Serialize)]
struct SpawnResponse {
    session_id: Option<String>,
    message: String,
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
        "Spawn request: agent_type={}, create_session={}",
        payload.agent_type, payload.create_session
    );

    // Create session if requested
    let session_id = if payload.create_session {
        let metadata = state.session_store.create_session(payload.agent_type.clone()).await?;
        Some(metadata.session_id)
    } else {
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
    let (child, mut rx) = state.agent_runner.spawn(agent_request).await?;

    // Store the process if we have a session_id
    if let Some(ref sid) = session_id {
        state.running_processes.lock().await.insert(sid.clone(), child);
    }

    // Create SSE stream
    let stream = async_stream::stream! {
        // Send initial session info if available
        if let Some(ref sid) = session_id {
            let event = Event::default()
                .json_data(serde_json::json!({
                    "type": "session_created",
                    "session_id": sid
                }))
                .unwrap();
            yield Ok(event);
        }

        // Stream JSONL lines from claude
        while let Some(result) = rx.recv().await {
            match result {
                Ok(line) => {
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "output",
                            "data": line
                        }))
                        .unwrap();
                    yield Ok(event);
                }
                Err(e) => {
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

        // Send completion event
        let event = Event::default()
            .json_data(serde_json::json!({
                "type": "completed"
            }))
            .unwrap();
        yield Ok(event);

        // Clean up running process
        if let Some(ref sid) = session_id {
            state.running_processes.lock().await.remove(sid);
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
    info!("Message request: session_id={}", session_id);

    // Verify session exists and update last_used
    state.session_store.touch_session(&session_id).await?;

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
    let (child, mut rx) = state.agent_runner.spawn(agent_request).await?;

    // Store the process
    state.running_processes.lock().await.insert(session_id.clone(), child);

    // Create SSE stream
    let stream = async_stream::stream! {
        // Stream JSONL lines from claude
        while let Some(result) = rx.recv().await {
            match result {
                Ok(line) => {
                    let event = Event::default()
                        .json_data(serde_json::json!({
                            "type": "output",
                            "data": line
                        }))
                        .unwrap();
                    yield Ok(event);
                }
                Err(e) => {
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

        // Send completion event
        let event = Event::default()
            .json_data(serde_json::json!({
                "type": "completed"
            }))
            .unwrap();
        yield Ok(event);

        // Clean up running process
        state.running_processes.lock().await.remove(&session_id);
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Terminate endpoint - forcefully terminate a running agent process
async fn terminate(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<impl IntoResponse> {
    info!("Terminate request: session_id={}", session_id);

    let mut processes = state.running_processes.lock().await;
    
    if let Some(child) = processes.remove(&session_id) {
        AgentRunner::terminate(child).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "message": "Process terminated successfully"
            })),
        ))
    } else {
        Err(AppError::SessionNotFound(session_id))
    }
}

/// List sessions endpoint
async fn list_sessions(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
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
    
    info!("Server listening on http://{}", addr);
    
    axum::serve(listener, app).await?;

    Ok(())
}
