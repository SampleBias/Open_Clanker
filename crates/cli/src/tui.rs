//! TUI client for Open Clanker Gateway
//!
//! Connects to a running gateway via HTTP (health) and WebSocket (events).

use anyhow::Result;
use futures_util::StreamExt; // needed for WebSocketStream::next()
use tokio_tungstenite::tungstenite::Message as WsMessage;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use serde::Deserialize;
use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Health response from gateway /health endpoint
#[derive(Debug, Clone, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub active_connections: usize,
    pub total_messages: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub timestamp: Option<String>,
}

/// Shared application state for TUI
#[derive(Debug, Clone, Default)]
pub struct TuiState {
    pub gateway_url: String,
    pub health: Option<HealthResponse>,
    pub events: Vec<String>,
    pub connection_status: String,
    pub error: Option<String>,
}

impl TuiState {
    pub fn new(gateway_url: String) -> Self {
        Self {
            gateway_url,
            health: None,
            events: Vec::new(),
            connection_status: "Connecting...".to_string(),
            error: None,
        }
    }

    pub fn add_event(&mut self, msg: String) {
        self.events.push(msg);
        // Keep last 100 events
        if self.events.len() > 100 {
            self.events.remove(0);
        }
    }

    pub fn set_connected(&mut self) {
        self.connection_status = "Connected".to_string();
        self.error = None;
    }

    pub fn set_disconnected(&mut self, reason: String) {
        self.connection_status = format!("Disconnected: {}", reason);
        self.error = Some(reason);
    }
}

/// Fetch health from gateway
async fn fetch_health(base_url: &str) -> Result<HealthResponse> {
    let url = format!("{}/health", base_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let resp = client.get(&url).send().await?;
    resp.error_for_status_ref()?;
    let health: HealthResponse = resp.json().await?;
    Ok(health)
}

/// Run the TUI
pub async fn run_tui(host: &str, port: u16) -> Result<()> {
    let base_url = format!("http://{}:{}", host, port);
    let ws_url = format!("ws://{}:{}/ws", host, port);

    let state = Arc::new(RwLock::new(TuiState::new(base_url.clone())));

    // Spawn health polling task
    let state_health = state.clone();
    let base_url_health = base_url.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        loop {
            interval.tick().await;
            match fetch_health(&base_url_health).await {
                Ok(health) => {
                    let mut s = state_health.write().await;
                    s.health = Some(health);
                    s.set_connected();
                }
                Err(e) => {
                    let mut s = state_health.write().await;
                    s.set_disconnected(e.to_string());
                }
            }
        }
    });

    // Spawn WebSocket task for events
    let state_ws = state.clone();
    let ws_url_clone = ws_url.clone();
    tokio::spawn(async move {
        loop {
            match tokio_tungstenite::connect_async(&ws_url_clone).await {
                Ok((mut ws_stream, _)) => {
                    {
                        let mut s = state_ws.write().await;
                        s.add_event("WebSocket connected".to_string());
                    }
                    while let Some(msg) = ws_stream.next().await {
                        match msg {
                            Ok(WsMessage::Text(text)) => {
                                let mut s = state_ws.write().await;
                                s.add_event(format!("WS: {}", text.chars().take(80).collect::<String>()));
                            }
                            Ok(WsMessage::Ping(_)) | Ok(WsMessage::Pong(_)) => {}
                            Ok(WsMessage::Close(_)) => break,
                            Err(_) => break,
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    let mut s = state_ws.write().await;
                    s.add_event(format!("WebSocket error: {}", e));
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Need to use futures_util::StreamExt for next()
    use futures_util::StreamExt;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let mut terminal = ratatui::Terminal::new(CrosstermBackend::new(stdout))?;

    let result = run_ui_loop(&mut terminal, state).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_ui_loop(
    terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>,
    state: Arc<RwLock<TuiState>>,
) -> Result<()> {
    loop {
        let state_read = state.read().await;
        draw_ui(terminal, &*state_read)?;
        drop(state_read);

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

fn draw_ui(
    terminal: &mut ratatui::Terminal<CrosstermBackend<Stdout>>,
    state: &TuiState,
) -> Result<()> {
    terminal.draw(|frame| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Status pane
        let status_block = Block::default()
            .title(" Gateway Status ")
            .borders(Borders::ALL);

        let status_text = if let Some(health) = &state.health {
            format!(
                "Status: {} | Version: {} | Uptime: {}s | Connections: {} | Messages: {}",
                health.status,
                health.version,
                health.uptime_seconds,
                health.active_connections,
                health.total_messages
            )
        } else {
            "Fetching...".to_string()
        };

        let status = Paragraph::new(status_text)
            .block(status_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(status, chunks[0]);

        // Events pane
        let events_block = Block::default()
            .title(" Events ")
            .borders(Borders::ALL);

        let events: Vec<ListItem> = state
            .events
            .iter()
            .rev()
            .take(20)
            .map(|e| ListItem::new(e.as_str()))
            .collect();

        let events_list = List::new(events).block(events_block);
        frame.render_widget(events_list, chunks[1]);

        // Instructions pane
        let help_block = Block::default()
            .title(" Help ")
            .borders(Borders::ALL);

        let help_text = if state.error.is_some() {
            format!(
                "Gateway: {} | {} | Press 'q' or Esc to quit",
                state.gateway_url, state.connection_status
            )
        } else {
            format!(
                "Gateway: {} | {} | Press 'q' or Esc to quit",
                state.gateway_url, state.connection_status
            )
        };

        let help = Paragraph::new(help_text)
            .block(help_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(help, chunks[2]);
    })?;

    Ok(())
}
