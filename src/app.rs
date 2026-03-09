use ratatui::widgets::ListState;
use std::collections::{BTreeMap, VecDeque};

use crate::mqtt;

const MAX_MESSAGES: usize = 200;

pub struct App {
    pub topic_tree: TopicNode,
    pub message_count: u64,
    pub connected: bool,
    pub selected: usize,
    pub list_state: ListState,
}

pub struct TopicNode {
    pub messages: VecDeque<mqtt::Message>,
    pub children: BTreeMap<String, TopicNode>,
    pub total_count: u64,
    pub expanded: bool,
}

pub struct TopicNodeFlat {
    pub depth: usize,
    pub label: String,
    pub has_children: bool,
    pub expanded: bool,
    pub message_count: u64,
    pub sub_topic_count: u64,
}

impl App {
    pub fn new() -> Self {
        Self {
            topic_tree: TopicNode::new(),
            message_count: 0,
            connected: false,
            selected: 0,
            list_state: ListState::default(),
        }
    }

    pub fn on_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.list_state.select(Some(self.selected));
    }

    pub fn on_down(&mut self, max: usize) {
        self.selected = (self.selected + 1).min(max.saturating_sub(1));
        self.list_state.select(Some(self.selected));
    }

    pub fn on_enter(&mut self) {
        let mut idx = self.selected;
        self.topic_tree.toggle_expanded(&mut idx);
    }

    pub fn on_message(&mut self, msg: mqtt::Message) {
        self.message_count += 1;
        self.topic_tree.insert(&msg.topic.clone(), msg);
    }

    pub fn on_connected(&mut self) {
        self.connected = true;
    }

    pub fn on_disconnected(&mut self) {
        self.connected = false;
    }

    pub fn selected_node(&self) -> Option<&TopicNode> {
        let mut idx = self.selected;
        self.topic_tree.get_node_at(&mut idx)
    }
}

impl TopicNode {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            children: BTreeMap::new(),
            total_count: 0,
            expanded: false,
        }
    }

    pub fn insert(&mut self, topic: &str, msg: mqtt::Message) {
        self.total_count += 1;

        let segments: Vec<&str> = topic.split('/').collect();
        let mut current = self;

        for seg in &segments {
            current = current
                .children
                .entry(seg.to_string())
                .or_insert_with(TopicNode::new);
            current.total_count += 1;
        }

        current.messages.push_back(msg);

        if current.messages.len() > MAX_MESSAGES {
            current.messages.pop_front();
        }
    }

    pub fn flatten(&self, target: &mut Vec<TopicNodeFlat>, depth: usize) {
        for (key, val) in self.children.iter() {
            target.push(TopicNodeFlat {
                depth,
                label: key.to_string(),
                has_children: !val.children.is_empty(),
                expanded: val.expanded,
                message_count: val.total_count,
                sub_topic_count: 42,
            });
            if val.expanded {
                val.flatten(target, depth + 1)
            }
        }
    }

    pub fn toggle_expanded(&mut self, counter: &mut usize) -> bool {
        for (_, val) in self.children.iter_mut() {
            if *counter == 0 {
                val.expanded = !val.expanded;
                return true;
            }
            *counter -= 1;

            if val.expanded && val.toggle_expanded(counter) {
                return true;
            }
        }
        false
    }

    pub fn visible_count(&self) -> usize {
        self.children
            .values()
            .map(|v| 1 + if v.expanded { v.visible_count() } else { 0 })
            .sum()
    }

    pub fn get_node_at(&self, position: &mut usize) -> Option<&TopicNode> {
        for (_, val) in self.children.iter() {
            if *position == 0 {
                return Some(val);
            }
            *position -= 1;

            if val.expanded {
                if let Some(node) = val.get_node_at(position) {
                    return Some(node);
                }
            }
        }
        None
    }
}
