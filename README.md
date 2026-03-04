# coingecko-realtime-websocket

> A Rust backend that polls CoinGecko for live crypto prices and streams normalized race state to connected clients over WebSocket.

![Rust](https://img.shields.io/badge/built%20with-Rust-orange?logo=rust)
![Tokio](https://img.shields.io/badge/async-Tokio-blue)
![Axum](https://img.shields.io/badge/server-Axum-purple)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Overview

Polls CoinGecko's `/simple/price` REST endpoint every 30 seconds and maintains a rolling 24-hour price history per coin. Every 500ms, it computes percentage changes over a configurable time window, normalizes them into speed values, accumulates car positions, and broadcasts the result as JSON to all connected WebSocket clients.

```
CoinGecko REST API
    │
    │ poll every 30s
    ▼
coingecko.rs ──► PriceStore ──► PriceEngine ──► Broadcast Loop ──► WebSocket clients
```

Originally built as the backend for [CryptoGP](https://github.com/yourusername/CryptoGP), a Formula 1-style crypto race visualizer.

---

## How It Works

1. **Polling** — A Tokio task hits `/simple/price` every 30s, fetching all configured coins in a single request.
2. **Storage** — Each price tick is appended to a `VecDeque` per coin and pruned to 24 hours.
3. **Percent change** — Finds the oldest tick within the selected time window; falls back to the oldest available tick during startup.
4. **Speed normalization** — Maps percent changes linearly to `[0.3, 1.5]` across all coins. Equal movers all get `0.9`.
5. **Position accumulation** — Every 500ms: `pos += speed × dt`, wraps at `1.0`.
6. **Broadcast** — Serialized `RaceState` JSON is pushed to all clients. Dead connections are pruned via `retain`.

---

## WebSocket Protocol

Connect to `ws://localhost:9001/ws`. The server pushes a message every 500ms.

**Payload:**

```json
{
  "timestamp": "2024-01-15T10:30:00.123Z",
  "time_window": "5m",
  "cars": [
    {
      "symbol": "bitcoin",
      "display_name": "Bitcoin",
      "price": 63241.50,
      "percent_change": 2.34,
      "speed": 1.12,
      "position": 0.73,
      "color_hex": "#e63946"
    }
  ]
}
```

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | ISO 8601 | Server time of computation |
| `time_window` | `"1m"` \| `"5m"` \| `"15m"` \| `"1h"` \| `"24h"` | Active price window |
| `percent_change` | `f64` | Relative to window open; `+` up, `-` down |
| `speed` | `f64` | Normalized to `[0.3, 1.5]` |
| `position` | `f64` | `0.0` = start, wraps at `1.0` (lapping) |

---

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs) (stable, 2021 edition)
- A [CoinGecko Demo API key](https://www.coingecko.com/en/api/pricing) (free)

### Run

```bash
cp .env.example .env
# Set COINGECKO_API_KEY in .env
cargo run
```

Server starts on `http://localhost:9001`.

### Test the WebSocket

```js
const ws = new WebSocket('ws://localhost:9001/ws');
ws.onmessage = e => console.log(JSON.parse(e.data));
```

Data arrives every 500ms. Prices populate after the first CoinGecko poll (~30s).

---

## Configuration

All settings via `.env`:

```env
PORT=9001
COIN_IDS=bitcoin,ethereum,solana,binancecoin,cardano,dogecoin,ripple,avalanche-2,polkadot,chainlink
DEFAULT_TIME_WINDOW=5m
BROADCAST_INTERVAL_MS=500
POLL_INTERVAL_MS=30000
COINGECKO_API_KEY=your-demo-key-here
```

---

## Project Structure

```
src/
├── main.rs           # Entry point — spawns poller, broadcaster, HTTP server
├── config.rs         # Config struct loaded from .env
├── models.rs         # PriceTick, CarState, RaceState (serde)
├── coingecko.rs      # REST poller with 429 handling + exponential backoff
├── price_engine.rs   # PriceStore, rolling windows, speed normalization
└── ws_server.rs      # Axum router, /ws handler, client registry
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` (full) | Async runtime |
| `axum` (ws) | HTTP + WebSocket server |
| `reqwest` (json) | CoinGecko HTTP client |
| `tower-http` (cors) | CORS middleware |
| `serde` / `serde_json` | JSON serialization |
| `futures-util` | WS sink/stream splitting |
| `dotenvy` | `.env` loading |
| `tracing` / `tracing-subscriber` | Structured logging |
| `chrono` (serde) | Timestamps + rolling window math |

## License

MIT