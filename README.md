# lng-mqtt-blitz

A small, high‑concurrency MQTT stress tester written in Rust. It spawns a configurable number of clients, publishes data at a regular interval, and displays live statistics in a terminal UI (TUI). Useful for benchmarking and load testing MQTT brokers.

---

## 🚀 Features

- Spawn hundreds or thousands of concurrent MQTT clients
- Configurable target, credentials, TLS, payload template, interval, and ramp-up
- Live terminal UI with throughput and connection progress
- Clean shutdown on `Ctrl-C` or pressing `q`
- Simple configuration via CLI flags, environment variables, or a `lng-config.toml` file

---

## 🛠️ Building

Requires Rust 1.65+ (stable).

```bash
git clone https://github.com/ShingShen/lng-mqtt-blitz
cd lng-mqtt-blitz
cargo build --release
```

Binaries will appear in `target/release/lng-mqtt-blitz`.

---

## ⚙️ Configuration

Configuration can come from three sources in order of precedence:

1. Command‑line arguments
2. Environment variables (prefixed with `LNG_`)
3. `lng-config.toml` file in the current directory (optional)

### Example `lng-config.toml`

```toml
# target broker
target_host = "mqtt.mybroker.local"
port = 8883
use_tls = true
username = "user"
password = "secret"

# load generation
connections = 500
interval_ms = 200
ramp_up_rate = 50

# payload (must include {{id}} and optional {{random}})
payload_template = '{"id": {{id}}, "temp": {{random}}}'
```

### Environment variables

You can export any of the same settings with the `LNG_` prefix. For example:

```bash
export LNG_BROKER_URL=broker.example.com
export LNG_CONNECTIONS=1000
export LNG_INTERVAL_MS=500
```

### Command-line options

Run `lng-mqtt-blitz --help` for the full list. Some common flags:

```
  -h, --help               Print help information
  -V, --version            Print version information
  -t, --target-host <HOST> MQTT broker host (env LNG_BROKER_URL)
  -p, --port <PORT>        MQTT broker port (env LNG_PORT)
      --use-tls             Enable TLS (env LNG_USE_TLS)
  -c, --connections <N>    Number of clients to spawn (env LNG_CONNECTIONS)
      --interval-ms <MS>   Publish interval in milliseconds (env LNG_INTERVAL_MS)
      --payload-template <TEMPLATE>
                           Payload format string (env LNG_PAYLOAD_TEMPLATE)
      --ramp-up-rate <N>   Clients to start per second (env LNG_RAMP_UP_RATE)
```

---

## ▶️ Running

```bash
# simple run against local broker
cargo run --release -- --connections 200 --interval-ms 100
```

or using the config file:

```bash
./target/release/lng-mqtt-blitz
```

The TUI appears immediately. Press **q** or **Ctrl-C** to exit cleanly.

---

## 🧩 TUI Controls

- **`q`** – quit the program
- **`Ctrl+C`** – also quits and triggers cleanup

The UI shows:

- Connection progress gauge
- Sent / received / errors counters
- Messages per second
- Active clients out of target count

---

## 🧪 Example Payload

By default each client publishes to `lng/blitz/<id>` with a JSON payload such as:

```json
{"id": 42, "temp": 73}
```

Use the `--payload-template` flag to customize; `{{id}}` is replaced with the
client index and `{{random}}` with a pseudo‑random integer (10–99).

---

## 📂 License & Contribution

This project is released under the MIT/Apache-2.0 dual license. Contributions are
welcome! Feel free to open issues or pull requests.
