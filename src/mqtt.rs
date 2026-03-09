use crate::args::Args;
use crate::config::Session;
use rumqttc::{AsyncClient, EventLoop, MqttOptions, QoS};
use std::time::Duration;

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

impl ClientConfig {
    pub fn from_args(args: &Args) -> Self {
        Self {
            hostname:  args.hostname.clone().unwrap_or_else(|| "localhost".to_string()),
            port:      args.port.unwrap_or(1883),
            client_id: args.client_id.clone().unwrap_or_else(|| "rmqtty-client".to_string()),
            user:      args.user.clone(),
            password:  args.password.clone(),
            topic:     args.topic.clone().unwrap_or_else(|| "#".to_string()),
        }
    }

    pub fn from_session(s: &Session) -> Self {
        Self {
            hostname:  s.host.clone(),
            port:      s.port.unwrap_or(1883),
            client_id: "rmqtty-client".to_string(),
            user:      s.user.clone(),
            password:  s.password.clone(),
            topic:     s.topics.as_ref()
                        .and_then(|t| t.first().cloned())
                        .unwrap_or_else(|| "#".to_string()),
        }
    }

    pub fn apply_cli_overrides(&mut self, args: &Args) {
        if let Some(h)  = &args.hostname  { self.hostname  = h.clone(); }
        if let Some(p)  = args.port       { self.port      = p; }
        if let Some(t)  = &args.topic     { self.topic     = t.clone(); }
        if let Some(u)  = &args.user      { self.user      = Some(u.clone()); }
        if let Some(pw) = &args.password  { self.password  = Some(pw.clone()); }
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
}
