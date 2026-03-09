use crate::app::App;
use chrono::Local;
use clap::Parser;
use crossterm::{
    ExecutableCommand,
    event::{Event, EventStream, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{Terminal, prelude::CrosstermBackend};
use rumqttc::{Event as RumqttEvent, Packet};
use std::{io::stdout, time::Duration};
use tokio::{select, sync::mpsc};

mod app;
mod args;
mod config;
mod mqtt;
mod ui;

pub enum MqttEvent {
    Connected,
    Publish(app::Message),
    Disconnected,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = args::Args::parse();

    let config = if let Some(name) = &cli_args.profile {
        let session = config::Config::load()
            .and_then(|cfg| cfg.get_sessions(name).ok().cloned());
        match session {
            Some(s) => {
                let mut cfg = mqtt::ClientConfig::from_session(&s);
                cfg.apply_cli_overrides(&cli_args);
                cfg
            }
            None => mqtt::ClientConfig::from_args(&cli_args),
        }
    } else {
        mqtt::ClientConfig::from_args(&cli_args)
    };

    let mut client = mqtt::Client::new(&config);
    client.subscribe_to_topic(&config.topic).await;
    let (tx, mut rx) = mpsc::unbounded_channel::<MqttEvent>();

    let mut eventloop = client.eventloop;

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(RumqttEvent::Incoming(Packet::ConnAck(_))) => {
                    let _ = tx.send(MqttEvent::Connected);
                }
                Ok(RumqttEvent::Incoming(Packet::Publish(packet))) => {
                    let msg = app::Message {
                        topic: packet.topic,
                        ts: Local::now(),
                        qos: packet.qos as u8,
                        retain: packet.retain,
                        payload: String::from_utf8_lossy(&packet.payload).to_string(),
                    };
                    let _ = tx.send(MqttEvent::Publish(msg));
                }
                Err(_) => {
                    let _ = tx.send(MqttEvent::Disconnected);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {}
            }
        }
    });

    let mut event_stream = EventStream::new();

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = app::App::new();

    loop {
        // Render
        terminal.draw(|frame| ui::draw(frame, &mut app))?;

        select! {
          key_event = event_stream.next() => {
              if let Some(Ok(Event::Key(key))) = key_event
                  && handle_key_event(key, &mut app) { break; }
          }
          Some(evt) = rx.recv() => {
              handle_mqtt_event(evt, &mut app);
          }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn handle_key_event(key: KeyEvent, app: &mut App) -> bool {
    let item_len = app.topic_tree.visible_count();
    match key.code {
        KeyCode::Char('q') => return true,
        KeyCode::Up => app.on_up(),
        KeyCode::Down => app.on_down(item_len),
        KeyCode::Enter => app.on_enter(),
        KeyCode::Char('j') => app.on_down(item_len),
        KeyCode::Char('k') => app.on_up(),
        // ... rest of keybindings
        _ => {}
    }
    false
}
fn handle_mqtt_event(evt: MqttEvent, app: &mut App) {
    match evt {
        MqttEvent::Connected => app.on_connected(),
        MqttEvent::Disconnected => app.on_disconnected(),
        MqttEvent::Publish(msg) => app.on_message(msg),
    }
}
