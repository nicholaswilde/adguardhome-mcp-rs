use axum::http::StatusCode;
use axum::{
    Json, Router,
    extract::{Query, Request as AxumRequest, State},
    middleware::{self, Next},
    response::{
        IntoResponse, Response as AxumResponse,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use dashmap::DashMap;
use futures::stream::Stream;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::mcp::{Request, Response, ResponseError};
use crate::server::mcp::McpServer;

#[derive(Clone)]
struct AppState {
    mcp_server: McpServer,
    sessions: Arc<DashMap<String, mpsc::Sender<Result<Event, Infallible>>>>,
    auth_token: Option<String>,
}

#[derive(Deserialize)]
struct MessageParams {
    session_id: String,
}

pub async fn run_http_server(
    mcp_server: McpServer,
    mut rx: mpsc::Receiver<crate::mcp::Notification>,
    host: &str,
    port: u16,
    auth_token: Option<String>,
) -> anyhow::Result<()> {
    let sessions = Arc::new(DashMap::new());
    let state = AppState {
        mcp_server,
        sessions: sessions.clone(),
        auth_token,
    };

    let app = create_router_with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("Starting HTTP MCP Server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Spawn notification handler
    tokio::spawn(async move {
        while let Some(n) = rx.recv().await {
            if let Ok(data) = serde_json::to_string(&crate::mcp::Message::Notification(n)) {
                for entry in sessions.iter() {
                    let tx = entry.value();
                    let _ = tx
                        .send(Ok(Event::default().event("message").data(data.clone())))
                        .await;
                }
            }
        }
    });

    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router(mcp_server: McpServer, auth_token: Option<String>) -> Router {
    let state = AppState {
        mcp_server,
        sessions: Arc::new(DashMap::new()),
        auth_token,
    };
    create_router_with_state(state)
}

fn create_router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let session_id = Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::channel(100);

    state.sessions.insert(session_id.clone(), tx.clone());

    info!("New SSE session connected: {}", session_id);

    // Send the endpoint event immediately
    let endpoint_url = format!("/message?session_id={}", session_id);
    let _ = tx
        .send(Ok(Event::default().event("endpoint").data(endpoint_url)))
        .await;

    // Create a stream that removes the session on drop
    let stream = ReceiverStream::new(rx);

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn message_handler(
    State(state): State<AppState>,
    Query(params): Query<MessageParams>,
    Json(req): Json<Request>,
) -> impl IntoResponse {
    let session_id = params.session_id;

    let tx = if let Some(sender) = state.sessions.get(&session_id) {
        sender.clone()
    } else {
        return (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response();
    };

    let mcp = state.mcp_server.clone();

    tokio::spawn(async move {
        let req_id = req.id.clone();
        debug!(
            "Received HTTP request for session {}: {:?}",
            session_id, req
        );

        let resp = mcp.handle_request(req).await;

        let json_resp = match resp {
            Ok(result) => Response {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: Some(result),
                error: None,
            },
            Err(e) => Response {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result: None,
                error: Some(ResponseError {
                    code: -32603,
                    message: e.to_string(),
                    data: None,
                }),
            },
        };

        if let Ok(data) = serde_json::to_string(&json_resp) {
            // Send response as 'message' event
            if let Err(e) = tx
                .send(Ok(Event::default().event("message").data(data)))
                .await
            {
                error!("Failed to send SSE event to session {}: {}", session_id, e);
            }
        }
    });

    // Return 202 Accepted immediately
    (axum::http::StatusCode::ACCEPTED, "Accepted").into_response()
}

async fn auth_middleware(
    State(state): State<AppState>,
    req: AxumRequest,
    next: Next,
) -> Result<AxumResponse, StatusCode> {
    if let Some(ref token) = state.auth_token {
        // 1. Check Header
        if let Some(auth_str) = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            && auth_str == format!("Bearer {}", token)
        {
            return Ok(next.run(req).await);
        }

        // 2. Check Query Param
        if let Some(query) = req.uri().query() {
            let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect();

            if params.get("token") == Some(token) {
                return Ok(next.run(req).await);
            }
        }

        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}
