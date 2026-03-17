use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::app::{App, TopicNodeFlat};
use serde_json;

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

    let msg_lines = app
        .selected_node()
        .and_then(|node| node.messages.back())
        .map(|m| {
            let mut lines = vec![
                Line::from(Span::styled(
                    format!(" {}", m.ts.format("%H:%M:%S")),
                    Style::default().fg(Color::DarkGray),
                )),
                Line::raw(""),
            ];
            lines.extend(format_payload(&m.payload));
            lines
        })
        .unwrap_or_default();

    let msg_widget = Paragraph::new(Text::from(msg_lines))
        .block(Block::default().borders(Borders::ALL).title("Message"))
        .wrap(Wrap { trim: false })
        .scroll((app.message_scroll, 0));

    frame.render_widget(msg_widget, bottom[1]);
}

fn format_payload(payload: &str) -> Vec<Line<'static>> {
    match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(v) => highlight_json(&v, 0),
        Err(_) => vec![Line::raw(payload.to_string())],
    }
}

fn scalar_span(value: &serde_json::Value) -> Span<'static> {
    match value {
        serde_json::Value::String(s) => {
            Span::styled(format!("\"{}\"", s), Style::default().fg(Color::Green))
        }
        serde_json::Value::Number(n) => {
            Span::styled(n.to_string(), Style::default().fg(Color::Cyan))
        }
        serde_json::Value::Bool(b) => {
            Span::styled(b.to_string(), Style::default().fg(Color::Magenta))
        }
        serde_json::Value::Null => Span::styled("null", Style::default().fg(Color::DarkGray)),
        _ => Span::raw(""),
    }
}

fn highlight_json(value: &serde_json::Value, indent: usize) -> Vec<Line<'static>> {
    let inner_pad = "  ".repeat(indent + 1);
    let close_pad = "  ".repeat(indent);

    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                return vec![Line::from(Span::raw("{}"))];
            }
            let mut lines = vec![Line::from(Span::raw("{"))];
            let len = map.len();
            for (i, (key, val)) in map.iter().enumerate() {
                let comma = if i < len - 1 { "," } else { "" };
                let key_span = Span::styled(
                    format!("{}\"{}\"", inner_pad, key),
                    Style::default().fg(Color::Yellow),
                );
                let colon = Span::raw(": ");
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        let mut sub = highlight_json(val, indent + 1);
                        if let Some(first) = sub.first_mut() {
                            first.spans.insert(0, colon);
                            first.spans.insert(0, key_span);
                        }
                        if let Some(last) = sub.last_mut() {
                            last.spans.push(Span::raw(comma));
                        }
                        lines.extend(sub);
                    }
                    _ => {
                        lines.push(Line::from(vec![
                            key_span,
                            colon,
                            scalar_span(val),
                            Span::raw(comma),
                        ]));
                    }
                }
            }
            lines.push(Line::from(Span::raw(format!("{}}}", close_pad))));
            lines
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return vec![Line::from(Span::raw("[]"))];
            }
            let mut lines = vec![Line::from(Span::raw("["))];
            let len = arr.len();
            for (i, val) in arr.iter().enumerate() {
                let comma = if i < len - 1 { "," } else { "" };
                match val {
                    serde_json::Value::Object(_) | serde_json::Value::Array(_) => {
                        let mut sub = highlight_json(val, indent + 1);
                        if let Some(first) = sub.first_mut() {
                            first.spans.insert(0, Span::raw(inner_pad.clone()));
                        }
                        if let Some(last) = sub.last_mut() {
                            last.spans.push(Span::raw(comma));
                        }
                        lines.extend(sub);
                    }
                    _ => {
                        lines.push(Line::from(vec![
                            Span::raw(inner_pad.clone()),
                            scalar_span(val),
                            Span::raw(comma),
                        ]));
                    }
                }
            }
            lines.push(Line::from(Span::raw(format!("{}]", close_pad))));
            lines
        }
        _ => vec![Line::from(scalar_span(value))],
    }
}
