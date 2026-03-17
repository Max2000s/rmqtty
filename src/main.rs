use crate::app::App;
use clap::Parser;
use crossterm::{
    ExecutableCommand,
    event::{Event, EventStream, KeyCode, KeyEvent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures_util::StreamExt;
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::io::stdout;
use tokio::{select, sync::mpsc};

mod app;
mod args;
mod config;
mod mqtt;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = args::Args::parse();

    let config = if let Some(name) = &cli_args.profile {
        let session = config::Config::load().and_then(|cfg| cfg.get_sessions(name).ok().cloned());
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
    let (tx, mut rx) = mpsc::unbounded_channel::<mqtt::MqttEvent>();
    client.start(tx);

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
        KeyCode::Esc => app.on_escape(),
        KeyCode::Char('j') => app.on_down(item_len),
        KeyCode::Char('k') => app.on_up(),
        KeyCode::Char('y') => app.on_yank(),
        // ... rest of keybindings
        _ => {}
    }
    false
}
fn handle_mqtt_event(evt: mqtt::MqttEvent, app: &mut App) {
    match evt {
        mqtt::MqttEvent::Connected => app.on_connected(),
        mqtt::MqttEvent::Disconnected => app.on_disconnected(),
        mqtt::MqttEvent::Publish(msg) => app.on_message(msg),
    }
}
