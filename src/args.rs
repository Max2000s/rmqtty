use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rmqtty", version, about = "Terminal based mqtt explorer", long_about = None)]
pub struct Args {
    /// Hostname of the mqtt broker
    #[arg(short = 'H', long, env = "RMQTTY_HOST")]
    pub hostname: Option<String>,

    /// Port of the broker
    #[arg(short = 'P', long, env = "RMQTTY_PORT")]
    pub port: Option<u16>,

    /// ClientID of the connection
    #[arg(short, long, env = "RMQTTY_CLIENT")]
    pub client_id: Option<String>,

    /// Topics to subscribe to
    #[arg(short, long, env = "RMQTTY_TOPIC")]
    pub topic: Option<String>,

    /// Username of the broker
    #[arg(short, long, env = "RMQTTY_USER")]
    pub user: Option<String>,

    /// Password of the broker
    #[arg(short, long, env = "RMQTTY_PW")]
    pub password: Option<String>,

    /// Selected profile from config.toml
    #[arg(long, env = "RMQTTY_PROFILE")]
    pub profile: Option<String>,
}
