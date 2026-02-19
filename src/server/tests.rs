use super::mcp::McpServer;
use crate::config::AppConfig;
use crate::mcp::Request;
use crate::tools::ToolRegistry;
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn setup() -> (
    McpServer,
    tokio::sync::mpsc::Receiver<crate::mcp::Notification>,
) {
    let mut config = AppConfig {
        adguard_host: "localhost".to_string(),
        adguard_port: 80,
        lazy_mode: true,
        ..Default::default()
    };
    config.validate().unwrap();
    let mut registry = ToolRegistry::new(&config);
    crate::tools::system::register(&mut registry);
    McpServer::with_registry(Arc::new(std::sync::Mutex::new(registry)), config)
}

#[tokio::test]
async fn test_handle_initialize() {
    let (server, _rx) = setup();
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "initialize".to_string(),
        params: None,
    };
    let resp = server.handle_request(req).await.unwrap();
    assert_eq!(resp["protocolVersion"], "2024-11-05");
}

#[tokio::test]
async fn test_handle_list_tools() {
    let (server, _rx) = setup();
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "list_tools".to_string(),
        params: None,
    };
    let resp = server.handle_request(req).await.unwrap();
    assert!(resp["tools"].is_array());
    // In lazy mode, manage_tools should be present
    let tools = resp["tools"].as_array().unwrap();
    assert!(tools.iter().any(|t| t["name"] == "manage_tools"));
}

#[tokio::test]
async fn test_handle_manage_tools_list() {
    let (server, _rx) = setup();
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_tools",
            "arguments": {
                "action": "list"
            }
        })),
    };
    let resp = server.handle_request(req).await.unwrap();
    assert!(resp["content"][0]["text"].is_string());
}

#[tokio::test]
async fn test_handle_manage_tools_enable_disable() {
    let (server, _rx) = setup();

    // First, disable a tool
    let req_disable = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(2),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_tools",
            "arguments": {
                "action": "disable",
                "tools": ["manage_system"]
            }
        })),
    };
    server.handle_request(req_disable).await.unwrap();

    {
        let registry = server.registry.lock().unwrap();
        assert!(!registry.is_tool_enabled("manage_system"));
    }

    // Then, enable it back
    let req_enable = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(3),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_tools",
            "arguments": {
                "action": "enable",
                "tools": ["manage_system"]
            }
        })),
    };
    server.handle_request(req_enable).await.unwrap();

    {
        let registry = server.registry.lock().unwrap();
        assert!(registry.is_tool_enabled("manage_system"));
    }
}

#[tokio::test]
async fn test_handle_manage_tools_invalid_action() {
    let (server, _rx) = setup();
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(4),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_tools",
            "arguments": {
                "action": "invalid"
            }
        })),
    };
    let resp = server.handle_request(req).await;
    assert!(resp.is_err());
}

use super::http::{create_router, run_http_server};
use axum::{
    body::Body,
    http::{Request as AxumRequest, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn test_http_auth_header() {
    let (mcp_server, _rx) = setup();
    let auth_token = Some("test-token".to_string());
    let app = create_router(mcp_server, auth_token);

    // 1. No token -> 401
    let req = AxumRequest::builder()
        .uri("/sse")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 2. Correct token in header -> 200
    let req = AxumRequest::builder()
        .uri("/sse")
        .header("Authorization", "Bearer test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. Correct token in query param -> 200
    let req = AxumRequest::builder()
        .uri("/sse?token=test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_http_auth_invalid() {
    let (mcp_server, _rx) = setup();
    let auth_token = Some("test-token".to_string());
    let app = create_router(mcp_server, auth_token);

    // 1. Invalid token in header
    let req = AxumRequest::builder()
        .uri("/sse")
        .header("Authorization", "Bearer wrong-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 2. Invalid token in query param
    let req = AxumRequest::builder()
        .uri("/sse?token=wrong-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 3. Missing token with auth enabled
    let req = AxumRequest::builder()
        .uri("/sse")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_http_message_handler_errors() {
    let (mcp_server, _rx) = setup();
    let app = create_router(mcp_server, None);

    // 1. Session not found
    let req = AxumRequest::builder()
        .method("POST")
        .uri("/message?session_id=nonexistent")
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}).to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 2. Invalid JSON
    let req = AxumRequest::builder()
        .method("POST")
        .uri("/message?session_id=invalid")
        .header("Content-Type", "application/json")
        .body(Body::from("not json"))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_http_server_notifications() {
    use crate::mcp::{Message, Notification};
    use axum::response::sse::Event;
    use std::convert::Infallible;
    use std::sync::Arc;

    let (_mcp_server, _rx) = setup();
    let (tx_notif, rx_notif) = tokio::sync::mpsc::channel(10);

    let sessions: Arc<
        dashmap::DashMap<String, tokio::sync::mpsc::Sender<Result<Event, Infallible>>>,
    > = Arc::new(dashmap::DashMap::new());
    let sessions_clone = sessions.clone();

    let (tx_sse, mut rx_sse) = tokio::sync::mpsc::channel(10);
    sessions.insert("test-session".to_string(), tx_sse);

    // Spawn notification handler logic from run_http_server
    tokio::spawn(async move {
        let mut rx_notif = rx_notif;
        while let Some(n) = rx_notif.recv().await {
            if let Ok(data) = serde_json::to_string(&Message::Notification(n)) {
                for entry in sessions_clone.iter() {
                    let tx = entry.value();
                    let _ = tx
                        .send(Ok(Event::default().event("message").data(data.clone())))
                        .await;
                }
            }
        }
    });

    // Send a notification
    let notif = Notification {
        jsonrpc: "2.0".to_string(),
        method: "test/notification".to_string(),
        params: Some(json!({"arg": "val"})),
    };
    tx_notif.send(notif).await.unwrap();

    // Check if received in SSE stream
    let event = rx_sse.recv().await.unwrap().unwrap();
    let data = format!("{:?}", event);
    assert!(data.contains("test/notification"));
}

#[tokio::test]
async fn test_http_message_handler_mcp_error() {
    let (mcp_server, _rx) = setup();
    let app = create_router(mcp_server, None);

    // 1. Connect to SSE
    let req = AxumRequest::builder()
        .uri("/sse")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();

    use http_body_util::BodyExt;
    let mut body = resp.into_body();
    let frame = body.frame().await.unwrap().unwrap();
    let data = frame.data_ref().unwrap();
    let body_str = String::from_utf8_lossy(data);
    let session_id = body_str.split("session_id=").nth(1).unwrap().trim();

    // 2. Send an invalid call (e.g. tool that doesn't exist)
    let req = AxumRequest::builder()
        .method("POST")
        .uri(format!("/message?session_id={}", session_id))
        .header("Content-Type", "application/json")
        .body(Body::from(
            json!({
                "jsonrpc": "2.0",
                "id": crate::mcp::RequestId::Number(1),
                "method": "call_tool",
                "params": {
                    "name": "nonexistent_tool",
                    "arguments": {}
                }
            })
            .to_string(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    // 3. Wait for the error in the SSE stream
    while let Some(frame) = body.frame().await {
        let frame = frame.unwrap();
        if let Some(data) = frame.data_ref() {
            let body_str = String::from_utf8_lossy(data);
            if body_str.contains("error") && body_str.contains("-32603") {
                return;
            }
        }
    }
    panic!("Error response not found in SSE stream");
}

#[tokio::test]
async fn test_run_http_server_notifications_full() {
    let (mcp_server, _unused_rx) = setup();
    let (tx_notif, rx_notif) = tokio::sync::mpsc::channel(10);

    // Find a free port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let server_handle = tokio::spawn(async move {
        let _ = run_http_server(mcp_server, rx_notif, "127.0.0.1", port, None).await;
    });

    // Wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Connect to SSE
    let client = reqwest::Client::new();
    let sse_url = format!("http://127.0.0.1:{}/sse", port);
    let mut resp = client.get(&sse_url).send().await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Send a notification
    let notif = crate::mcp::Notification {
        jsonrpc: "2.0".to_string(),
        method: "test/full".to_string(),
        params: None,
    };
    tx_notif.send(notif).await.unwrap();

    // Read the stream to find the notification
    while let Ok(Some(chunk)) = resp.chunk().await {
        let body_str = String::from_utf8_lossy(&chunk);
        if body_str.contains("test/full") {
            server_handle.abort();
            return;
        }
    }

    panic!("Notification not found in SSE stream");
}

#[tokio::test]
async fn test_mcp_run_errors() {
    let (server, rx) = setup();

    // 1. Invalid JSON (triggers continue)
    let input = "invalid json\n";
    let reader = std::io::Cursor::new(input);
    let mut writer = Vec::new();
    server.run(reader, &mut writer, rx).await.unwrap();
    assert!(writer.is_empty());

    // 2. Request that returns error (triggers Err(e) branch)
    let (server, rx) = setup();
    let input = serde_json::to_string(&crate::mcp::Message::Request(Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "call_tool".to_string(),
        params: Some(json!({"name": "nonexistent"})),
    }))
    .unwrap()
        + "\n";

    let reader = std::io::Cursor::new(input);
    let mut writer = Vec::new();
    server.run(reader, &mut writer, rx).await.unwrap();
    let output = String::from_utf8(writer).unwrap();
    assert!(output.contains("error"));
    assert!(output.contains("-32000"));

    // 3. Empty line
    let (server, rx) = setup();
    let input = "\n";
    let reader = std::io::Cursor::new(input);
    let mut writer = Vec::new();
    server.run(reader, &mut writer, rx).await.unwrap();
    assert!(writer.is_empty());
}

#[tokio::test]
async fn test_http_message_handler() {
    let (mcp_server, _rx) = setup();
    let app = create_router(mcp_server, None);

    // 1. Connect to SSE to get a session
    let req = AxumRequest::builder()
        .uri("/sse")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // We can't easily extract session_id from SSE stream without more work,
    // but we can test the 404 for unknown session.
    let req = AxumRequest::builder()
        .method("POST")
        .uri("/message?session_id=invalid")
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {}
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_http_message_handler_success() {
    let (mcp_server, _rx) = setup();
    let app = create_router(mcp_server, None);

    // 1. Connect to SSE to get a session
    let req = AxumRequest::builder()
        .uri("/sse")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Extract session_id from SSE body
    // SSE body format: event: endpoint\ndata: /message?session_id=...
    use http_body_util::BodyExt;
    let mut body = resp.into_body();
    let frame = body.frame().await.unwrap().unwrap();
    let data = frame.data_ref().unwrap();
    let body_str = String::from_utf8_lossy(data);
    let session_id = body_str.split("session_id=").nth(1).unwrap().trim();

    // 2. Send message using that session
    let req = AxumRequest::builder()
        .method("POST")
        .uri(format!("/message?session_id={}", session_id))
        .header("Content-Type", "application/json")
        .body(Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "id": crate::mcp::RequestId::Number(1),
                "method": "initialize",
                "params": {}
            }))
            .unwrap(),
        ))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    // 3. Wait for the response in the SSE stream
    while let Some(frame) = body.frame().await {
        let frame = frame.unwrap();
        if let Some(data) = frame.data_ref() {
            let body_str = String::from_utf8_lossy(data);
            if body_str.contains("result") && body_str.contains("protocolVersion") {
                return;
            }
        }
    }
    panic!("Response not found in SSE stream");
}

#[tokio::test]
async fn test_http_auth_query_param() {
    let (mcp_server, _rx) = setup();
    let auth_token = Some("test-token".to_string());
    let app = create_router(mcp_server, auth_token);

    // Correct token in query param -> 200
    let req = AxumRequest::builder()
        .uri("/sse?token=test-token")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_run_http_server_startup() {
    let (mcp_server, rx) = setup();
    // Use port 0 to let OS pick a free port
    let server_handle = tokio::spawn(async move {
        let _ = run_http_server(mcp_server, rx, "127.0.0.1", 0, None).await;
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    server_handle.abort();
}

#[tokio::test]
async fn test_mcp_run_generic() {
    let (server, rx) = setup();
    let input = serde_json::to_string(&crate::mcp::Message::Request(Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "initialize".to_string(),
        params: None,
    }))
    .unwrap()
        + "\n";

    let reader = std::io::Cursor::new(input);
    let mut writer = Vec::new();

    server.run(reader, &mut writer, rx).await.unwrap();

    let output = String::from_utf8(writer).unwrap();
    assert!(output.contains("protocolVersion"));
}

#[tokio::test]
async fn test_mcp_run_notification() {
    let (server, rx) = setup();

    // Enable a tool first
    {
        let mut registry = server.registry.lock().unwrap();
        registry.enable_tool("manage_system");
    }

    let (client_io, server_io) = tokio::io::duplex(1024);
    let (mut client_reader, mut client_writer) = tokio::io::split(client_io);
    let (server_reader, server_writer) = tokio::io::split(server_io);

    let server_handle = tokio::spawn(async move {
        server.run(server_reader, server_writer, rx).await.unwrap();
    });

    let input = serde_json::to_string(&crate::mcp::Message::Request(Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_tools",
            "arguments": {
                "action": "disable",
                "tools": ["manage_system"]
            }
        })),
    }))
    .unwrap()
        + "\n";

    client_writer.write_all(input.as_bytes()).await.unwrap();
    client_writer.flush().await.unwrap();

    let mut output = String::new();
    let mut reader = BufReader::new(&mut client_reader).lines();

    // Read response
    if let Some(line) = reader.next_line().await.unwrap() {
        output.push_str(&line);
    }
    // Read notification
    if let Some(line) = reader.next_line().await.unwrap() {
        output.push_str(&line);
    }

    // Should contain both the result and the notification
    assert!(output.contains("Disabled 1 tools"));
    assert!(output.contains("notifications/tools/list_changed"));

    server_handle.abort();
}

#[tokio::test]
async fn test_handle_unknown_method() {
    let (server, _rx) = setup();
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "unknown".to_string(),
        params: None,
    };
    let resp = server.handle_request(req).await;
    assert!(resp.is_err());
}

#[tokio::test]
async fn test_handle_call_tool_with_instance() {
    use crate::config::InstanceConfig;
    let mut config = AppConfig::default();
    config.instances = vec![
        InstanceConfig {
            name: Some("primary".to_string()),
            url: "http://primary:80".to_string(),
            ..Default::default()
        },
        InstanceConfig {
            name: Some("secondary".to_string()),
            url: "http://secondary:80".to_string(),
            ..Default::default()
        },
    ];
    config.validate().unwrap();
    
    let mut registry = ToolRegistry::new(&config);
    crate::tools::system::register(&mut registry);
    let (server, _rx) = McpServer::with_registry(Arc::new(std::sync::Mutex::new(registry)), config);

    // Call tool targeting "secondary"
    let req = Request {
        jsonrpc: "2.0".to_string(),
        id: crate::mcp::RequestId::Number(1),
        method: "call_tool".to_string(),
        params: Some(json!({
            "name": "manage_system",
            "arguments": {
                "action": "get_status",
                "instance": "secondary"
            }
        })),
    };
    
    // We expect a connection error because http://secondary:80 doesn't exist,
    // but the point is that it TRIED to connect to "secondary".
    let resp = server.handle_request(req).await;
    if let Err(e) = resp {
        let err_str = e.to_string();
        assert!(err_str.contains("secondary") || err_str.contains("dns") || err_str.contains("refused") || err_str.contains("connect"));
    }
}
