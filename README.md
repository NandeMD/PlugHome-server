# PlugHome-server
Open, local-first EV charging management for home users.

## Backend
- Rust + Tokio + Axum web stack; entrypoint lives in api/src/main.rs and binds to ADDR:PORT from .env via dotenvy.
- Exposes a health check at `/` and a WebSocket upgrader at `/:station_id` for charger sessions.
- Uses tracing for structured logs and installs a custom panic hook so crashes surface in logs.

## OCPP 1.6J WebSockets
- Built around rust-ocpp for parsing/serializing OCPP 1.6 JSON frames.
- Current handlers cover core calls such as Authorize, BootNotification, Heartbeat, DataTransfer, StatusNotification, and StopTransaction with example CallResult responses.
- Channel-specific state and message helpers live in the occp_ws crate (handlers, routes, state, types modules).
- The WebSocket flow and message handling are based on FlipSoftware/moovolt-mvp: https://github.com/FlipSoftware/moovolt-mvp

## Run locally
1) Create a .env file with ADDR (e.g., 0.0.0.0) and PORT (e.g., 3000).
2) Start the backend: `cargo run -p api`.
3) Connect an OCPP 1.6J client to ws://ADDR:PORT/{station_id} and exercise the core calls above.
