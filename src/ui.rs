use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, TopicNodeFlat};

pub fn draw(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(frame.area());

    let status = if app.connected {
        "Connected"
    } else {
        "Disconnected"
    };
    let status_color = if app.connected {
        Color::Green
    } else {
        Color::Red
    };

    let status_bar = Paragraph::new(format!(
        " Status: {}  |  Messages: {}",
        status, app.message_count
    ))
    .style(Style::default().fg(status_color))
    .block(Block::default().borders(Borders::ALL).title("rmqtty"));

    frame.render_widget(status_bar, chunks[0]);

    let mut flat_topics: Vec<TopicNodeFlat> = Vec::new();
    app.topic_tree.flatten(&mut flat_topics, 0);

    let items: Vec<_> = flat_topics
        .iter()
        .map(|node| {
            let indent = " ".repeat(node.depth);
            let indicator = match (node.expanded, node.has_children) {
                (true, true) => "▼",
                (false, true) => "▶",
                (_, false) => "·",
            };
            let text = format!(
                " {}{} {} | ({} msgs, {} topics)",
                indent, indicator, node.label, node.message_count, node.sub_topic_count
            );
            ListItem::new(text)
        })
        .collect();

    let topics_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Topics"))
        .highlight_style(Style::default().bg(Color::Blue));

    let bottom = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    frame.render_stateful_widget(topics_widget, bottom[0], &mut app.list_state);

    let msg_text = app
        .selected_node()
        .and_then(|node| node.messages.back())
        .map(|m| format!(" {} | {}", m.ts.format("%H:%M:%S"), m.payload))
        .unwrap_or_default();

    let msg_widget =
        Paragraph::new(msg_text).block(Block::default().borders(Borders::ALL).title("Messages"));

    frame.render_widget(msg_widget, bottom[1]);
}
