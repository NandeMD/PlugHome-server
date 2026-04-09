# PlugHome-server
Open, local-first EV charging management for home users.

## Backend
- Rust + Tokio + Axum web stack; entrypoint lives in api/src/main.rs and binds to ADDR:PORT from .env.
- Exposes a health check at `/` and a WebSocket upgrader at `/:station_id` for charger sessions.

## OCPP 1.6J WebSockets
- Built around rust-ocpp for parsing/serializing OCPP 1.6 JSON frames.
- Channel-specific state and message helpers live in the occp_ws crate (handlers, routes, state, types modules).
- The WebSocket flow and message handling are based on [FlipSoftware/moovolt-mvp](https://github.com/FlipSoftware/moovolt-mvp)

## Run locally
1) Create a .env file with ADDR (e.g., 0.0.0.0) and PORT (e.g., 3000).
  ```bash
  ADDR=0.0.0.0
  PORT=3000
  ```
2) Start the backend with: `cargo run api`
3) Connect an OCPP 1.6J client to ws://ADDR:PORT/{station_id}

---

## ✅ Planned Features & Capabilities

This project aims to provide a **self-hosted EV charger management server** that works reliably in home environments while remaining standards-compliant.

---

## 🎯 Project Goals (In Plain Terms)

* Run your **own charger server at home**
* Keep **full control of your charging data**
* Avoid cloud lock-in
* Work with real-world chargers reliably
* Stay simple first, powerful later

---

### 🔌 Charger Connectivity

* [x] Chargers can connect securely to the server
* [x] Automatic charger registration on first connection
* [x] Live online / offline status
* [x] Last-seen timestamp for each charger
* [x] Support for chargers behind NAT (outbound connection only)
* [ ] Graceful handling of reconnects and power loss

---

### ⚡ Charging Status & Monitoring

* [ ] View current charging state (available, charging, finished, error)
* [ ] View per-connector status
* [ ] Detect and display charger faults
* [ ] Real-time charging progress updates

---

### 🔋 Charging Sessions

* [ ] Automatically record each charging session
* [ ] Track start and stop time
* [ ] Track energy used per session
* [ ] Track charging stop reason (unplugged, remote stop, error)
* [ ] View charging history
* [ ] Export session data (JSON / CSV)

---

### 🎛️ Remote Control

* [ ] Start charging remotely
* [ ] Stop charging remotely
* [ ] Prevent commands when charger is offline
* [ ] Safe timeouts if charger does not respond
* [ ] Clear feedback when commands succeed or fail

---

### 🧠 Smart Charging (Home Focus)

* [ ] Set maximum charging current or power
* [ ] Schedule charging by time of day
* [ ] Pause charging outside allowed hours
* [ ] Manual override of schedules
* [ ] Remember preferred charging rules per charger

---

### 📊 Energy & Usage Insights

* [ ] Show total energy used per charger
* [ ] Show energy used per day / week / month
* [ ] Basic cost estimation (static electricity rates)
* [ ] Download usage data

---

### 🌐 REST API (Client Access)

* [ ] REST API for dashboards and mobile apps
* [ ] Read charger status and history
* [ ] Control chargers via API
* [ ] Simple authentication for clients
* [ ] Versioned API endpoints

---

### 🔐 Security & Safety

* [ ] Charger authentication using shared secrets
* [ ] Secure WebSocket connections (TLS)
* [ ] Optional LAN-only operation
* [ ] Prevent unauthorized control actions
* [ ] Audit log for control operations

---

### 🏠 Home Automation & Integrations

* [ ] Home Assistant integration
* [ ] MQTT support for status and control
* [ ] Webhook notifications for important events
* [ ] Easy integration with external energy systems

---

### 🛠️ Reliability & Maintenance

* [ ] Automatic detection of offline chargers
* [ ] Safe recovery after server restart
* [ ] Transaction recovery after reconnect
* [ ] Detailed logs for debugging
* [x] Health check endpoint

---

### 🔧 Developer & Operator Experience

* [ ] Single-binary deployment
* [ ] Minimal configuration for home users
* [ ] Clear documentation and examples
* [ ] Easy local testing with real chargers

---

### 📐 Standards Support

* [X] OCPP 1.6 JSON support
* [ ] OCPP 2.0.1 support
* [ ] Backward-compatible protocol handling
* [ ] Tolerant handling of vendor quirks

---
