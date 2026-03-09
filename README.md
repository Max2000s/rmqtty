# rmqtty

Terminal MQTT explorer.

## Run

```bash
cargo run -- [OPTIONS]
```

**Options**

| Flag | Env | Default | Description |
|------|-----|---------|-------------|
| `-H, --hostname` | `RMQTTY_HOST` | `localhost` | Broker hostname |
| `-P, --port` | `RMQTTY_PORT` | `1883` | Broker port |
| `-t, --topic` | `RMQTTY_TOPIC` | `#` | Topic to subscribe |
| `-u, --user` | `RMQTTY_USER` | | Username |
| `-p, --password` | `RMQTTY_PW` | | Password |
| `--profile` | `RMQTTY_PROFILE` | | Session profile name |

## Sessions

Define named sessions in `~/.config/rmqtty/config.toml`:

```toml
[sessions.home]
host = "192.168.1.10"
port = 1883
topics = ["home/#"]
user = "user"
password = "secret"
```

```bash
cargo run -- --profile home
```
