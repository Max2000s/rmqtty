use crate::args::Args;
use crate::config::Session;
use chrono::{DateTime, Local};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct ClientConfig {
    pub hostname: String,
    pub port: u16,
    pub client_id: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub topic: String,
}

pub struct Client {
    client: AsyncClient,
    pub eventloop: EventLoop,
}

pub struct Message {
    pub topic: String,
    pub ts: DateTime<Local>,
    pub qos: u8,
    pub retain: bool,
    pub payload: String,
}

pub enum MqttEvent {
    Connected,
    Publish(Message),
    Disconnected,
}

impl ClientConfig {
    pub fn from_args(args: &Args) -> Self {
        Self {
            hostname: args
                .hostname
                .clone()
                .unwrap_or_else(|| "localhost".to_string()),
            port: args.port.unwrap_or(1883),
            client_id: args
                .client_id
                .clone()
                .unwrap_or_else(|| "rmqtty-client".to_string()),
            user: args.user.clone(),
            password: args.password.clone(),
            topic: args.topic.clone().unwrap_or_else(|| "#".to_string()),
        }
    }

    pub fn from_session(s: &Session) -> Self {
        Self {
            hostname: s.host.clone(),
            port: s.port.unwrap_or(1883),
            client_id: "rmqtty-client".to_string(),
            user: s.user.clone(),
            password: s.password.clone(),
            topic: s
                .topics
                .as_ref()
                .and_then(|t| t.first().cloned())
                .unwrap_or_else(|| "#".to_string()),
        }
    }

    pub fn apply_cli_overrides(&mut self, args: &Args) {
        if let Some(h) = &args.hostname {
            self.hostname = h.clone();
        }
        if let Some(p) = args.port {
            self.port = p;
        }
        if let Some(t) = &args.topic {
            self.topic = t.clone();
        }
        if let Some(u) = &args.user {
            self.user = Some(u.clone());
        }
        if let Some(pw) = &args.password {
            self.password = Some(pw.clone());
        }
    }
}

impl Client {
    pub fn new(config: &ClientConfig) -> Client {
        let mut opts = MqttOptions::new(&config.client_id, &config.hostname, config.port);
        opts.set_keep_alive(Duration::from_secs(5));

        if let (Some(user), Some(pw)) = (&config.user, &config.password) {
            opts.set_credentials(user, pw);
        }

        let (client, eventloop) = AsyncClient::new(opts, 10);

        Client { client, eventloop }
    }

    pub async fn subscribe_to_topic(&mut self, topic: &str) {
        self.client.subscribe(topic, QoS::AtMostOnce).await.unwrap();
    }

    pub fn start(mut self, tx: mpsc::UnboundedSender<MqttEvent>) {
        tokio::spawn(async move {
            loop {
                match self.eventloop.poll().await {
                    Ok(Event::Incoming(Packet::ConnAck(_))) => {
                        let _ = tx.send(MqttEvent::Connected);
                    }
                    Ok(Event::Incoming(Packet::Publish(packet))) => {
                        let msg = Message {
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
    }
}
