use super::mcp::McpServer;
use crate::adguard::AdGuardClient;
use crate::config::AppConfig;
use crate::mcp::Request;
use crate::tools::ToolRegistry;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

fn setup() -> (
    McpServer,
    tokio::sync::mpsc::Receiver<crate::mcp::Notification>,
) {
    let config = AppConfig {
        adguard_host: "localhost".to_string(),
        adguard_port: 80,
        lazy_mode: true,
        ..Default::default()
    };
    let client = AdGuardClient::new(config.clone());
    let mut registry = ToolRegistry::new(&config);
    crate::tools::system::register(&mut registry);
    McpServer::new(client, registry, config)
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
