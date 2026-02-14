use crate::processor;
use crate::state::AppState;
use anyhow::Result;
use clanker_core::Message;
use crate::types::{HealthResponse, WsClientMessage, WsServerMessage};
use axum::{
    extract::{
        State,
        WebSocketUpgrade,
    },
    response::{IntoResponse, Json},
};
use axum::extract::ws::{Message as WsMessage, WebSocket, Utf8Bytes};
use futures_util::{SinkExt, StreamExt};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Health check handler
#[axum::debug_handler]
pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = state.uptime_seconds() as u64;
    let active_connections = state.connection_count().await;
    let total_messages = state.total_message_count();
    let active_workers = state.worker_count();
    let max_workers = state.worker_max();
    let version = state.version().to_string();

    let health = HealthResponse::new(
        version,
        uptime,
        active_connections,
        total_messages,
        active_workers,
        max_workers,
    );

    debug!(
        "Health check: {} connections, {} messages, {} workers",
        active_connections, total_messages, active_workers
    );

    Json(health)
}

/// Root handler
#[axum::debug_handler]
pub async fn root() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Open Clanker Gateway",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "AI Assistant Gateway with WebSocket support",
        "endpoints": {
            "health": "/health",
            "ws": "/ws"
        }
    }))
}

/// WebSocket handler
#[axum::debug_handler]
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    info!("New WebSocket connection requested");
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, state: AppState) {
    // Split WebSocket into sender and receiver
    let (mut sender, mut receiver) = socket.split();
    let connection_id = Uuid::new_v4();
    let mut conn_state = crate::types::ConnectionState::new(
        "0.0.0.0:0".parse().unwrap(),
    );
    conn_state.id = connection_id;

    info!("WebSocket connection {} established", connection_id);

    // Add connection to state
    state.add_connection(connection_id, conn_state.clone()).await;

    // Subscribe to broadcasts
    let mut broadcast_rx = state.broadcaster().subscribe();

    // Send welcome message
    let welcome = WsServerMessage::Health {
        status: "connected".to_string(),
        uptime_seconds: state.uptime_seconds() as u64,
    };

    if let Err(e) = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&welcome).unwrap()))).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Main event loop
    loop {
        tokio::select! {
            // Handle incoming client messages
            Some(result) = receiver.next() => {
                match result {
                    Ok(msg) => {
                        if let Err(e) = handle_client_message(msg, &state, &mut sender, &connection_id).await {
                            error!("Error handling client message: {}", e);

                            // Send error to client
                            let error_msg = WsServerMessage::error("MESSAGE_ERROR", e.to_string());
                            let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&error_msg).unwrap()))).await;
                        }
                    }
                    Err(e) => {
                        warn!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }

            // Handle broadcast messages
            result = broadcast_rx.recv() => {
                match result {
                    Ok(broadcast_msg) => {
                        // Filter messages for this connection's subscriptions
                        if should_send_to_message(&broadcast_msg, &conn_state) {
                            if let Err(e) = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&broadcast_msg).unwrap()))).await {
                                error!("Failed to send broadcast message: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Broadcast receive error: {}", e);
                        break;
                    }
                }
            }

            // Handle shutdown signal
            _ = state.shutdown_token().cancelled() => {
                info!("Shutdown signal received, closing connection {}", connection_id);
                break;
            }
        }
    }

    // Cleanup: remove connection
    state.remove_connection(&connection_id).await;
    info!("WebSocket connection {} closed", connection_id);
}

/// Handle client message
async fn handle_client_message(
    msg: WsMessage,
    state: &AppState,
    sender: &mut futures_util::stream::SplitSink<WebSocket, WsMessage>,
    connection_id: &Uuid,
) -> Result<(), anyhow::Error> {
    match msg {
        WsMessage::Text(text) => {
            // Parse JSON message
            let client_msg: WsClientMessage = serde_json::from_str(&text)?;

            match client_msg {
                WsClientMessage::Ping { timestamp } => {
                    // Respond with pong
                    let pong = WsServerMessage::Pong { timestamp };
                    let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&pong)?))).await;
                }

                WsClientMessage::Subscribe { channel_id, channel_type } => {
                    debug!("Connection {} subscribed to {} ({})", connection_id, channel_id, channel_type);

                    // Send confirmation
                    let sub_msg = WsServerMessage::Subscribed {
                        channel_id: channel_id.clone(),
                        connection_id: *connection_id,
                    };
                    let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&sub_msg)?))).await;
                }

                WsClientMessage::Unsubscribe { channel_id } => {
                    debug!("Connection {} unsubscribed from {}", connection_id, channel_id);

                    // Send confirmation
                    let unsub_msg = WsServerMessage::Unsubscribed {
                        channel_id: channel_id.clone(),
                    };
                    let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&unsub_msg)?))).await;
                }

                WsClientMessage::SendMessage { channel_id, channel_type, message } => {
                    debug!("Sending message to channel {} ({}): {}", channel_id, channel_type, message);

                    // Increment message count
                    state.increment_message_count();

                    // Build incoming message and process through agent
                    let incoming = Message::new(
                        channel_type,
                        channel_id.clone(),
                        "user".to_string(),
                        message,
                    );

                    match processor::process_message(&state, &incoming).await {
                        Ok(response_msg) => {
                            let response = WsServerMessage::send_response(
                                true,
                                Some(response_msg.id.clone()),
                                None,
                                Some(response_msg.text),
                            );
                            let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&response)?))).await;
                        }
                        Err(e) => {
                            let response = WsServerMessage::send_response(
                                false,
                                None,
                                Some(e),
                                None,
                            );
                            let _ = sender.send(WsMessage::Text(Utf8Bytes::from(serde_json::to_string(&response)?))).await;
                        }
                    }
                }
            }
        }

        WsMessage::Close(frame) => {
            info!("Client {} requested close: {:?}", connection_id, frame);
        }

        WsMessage::Ping(data) => {
            // Respond with pong
            let _ = sender.send(WsMessage::Pong(data)).await;
        }

        WsMessage::Pong(_) => {
            // Pong received, ignore
        }

        _ => {
            debug!("Received unhandled message type from {}", connection_id);
        }
    }

    Ok(())
}

/// Check if message should be sent to connection based on subscriptions
fn should_send_to_message(message: &WsServerMessage, conn_state: &crate::types::ConnectionState) -> bool {
    match message {
        WsServerMessage::MessageReceived(msg) => {
            // Send if connection is subscribed to this channel
            conn_state.is_subscribed(&msg.channel_id)
        }
        // Send all other message types
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let health = HealthResponse::new("1.0.0".to_string(), 100, 5, 1000, 2, 5);

        let json = serde_json::to_string(&health).unwrap();

        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("\"uptime_seconds\":100"));
        assert!(json.contains("\"active_connections\":5"));
        assert!(json.contains("\"total_messages\":1000"));
        assert!(json.contains("\"active_workers\":2"));
        assert!(json.contains("\"max_workers\":5"));
    }
}
