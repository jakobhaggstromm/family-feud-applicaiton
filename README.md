# Family Feud Buzzer

A real-time buzzer app for Family Feud-style games. Players connect via their browser and race to buzz in first. A host controls rounds from a separate panel.

## Architecture

- **Backend** — Rust (Axum) WebSocket server on port `3000`
- **Frontend** — React (Vite) dev server on port `5173`, proxies WebSocket to the backend

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) >= 18 and npm

## Start the Backend

```bash
cd server
cargo run
```

The server will start at `ws://0.0.0.0:3000/ws`.

## Start the Frontend

```bash
cd frontend
npm install
npm run dev
```

The app will be available at [http://localhost:5173](http://localhost:5173).

## Usage

1. Open **Admin** at [http://localhost:5173/admin](http://localhost:5173/admin).
2. Open **Player** devices at [http://localhost:5173/player](http://localhost:5173/player).
3. Players enter a team name and click **Join Game**.
4. Admin clicks **Start Game** (loads questions and begins play phase).
5. Players hit **BUZZ**; first buzzed team gets control, then admin judges with **Award** / **Strike**.

## Run Tests (Python)

```bash
cd test
pip install websockets
python test_server.py
```

Make sure the backend is running before executing the tests.
