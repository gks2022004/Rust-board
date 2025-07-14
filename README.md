# Realtime Collaborative Whiteboard

A real-time collaborative whiteboard built in **Rust** using:

* 🧠 `Yew` for the **frontend** (WASM-powered)
* 🌀 `Axum` WebSocket server for the **backend**

## Live Communication Architecture

This project leverages WebSockets to enable smooth, real-time drawing across multiple connected clients.

---

## Features

* 🎨 Freehand, Line, Rectangle, Circle, and Text drawing tools
* 📡 Real-time multi-user drawing sync
* 🔍 Zooming and Panning
* 📏 Adjustable stroke width
* 🎨 Color palette
* ✅ WebAssembly-powered frontend using `yew`

---

## Tech Stack

| Layer    | Tech                 |
| -------- | -------------------- |
| Frontend | Yew, WebAssembly     |
| Backend  | Axum, Tokio          |
| Shared   | Serde (event schema) |

---

## Screenshots

> <img width="1914" height="867" alt="image" src="https://github.com/user-attachments/assets/767ef762-e589-4887-9faf-31d43a59246d" />



## ▶️ Getting Started

### ✅ Prerequisites

* Rust + `wasm-pack`
* Node.js (for serving frontend if needed)
* `trunk` (or use `wasm-pack build && serve`)

### 🔧 Setup

```bash
# 1. Backend
cd backend
cargo run

# 2. Frontend (in another terminal)
cd frontend
trunk serve
```

Visit: `http://127.0.0.1:8080`

---

## 🔐 Event Format (WhiteboardEvent)

```json
{
  "type": "DrawLine",
  "from": [100, 200],
  "to": [400, 200],
  "color": "#2563eb",
  "width": 3.0
}
```

Other event types include: `DrawFreehand`, `DrawRect`, `DrawCircle`, `AddText`, `Pan`, `Zoom`

---

## 🤝 Contributing

Pull requests are welcome! Please include tests and update the documentation as needed.

---

## Credits

Created by Gaurav Kumar using Rust, Yew, and Axum ❤️

