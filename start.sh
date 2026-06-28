#!/bin/bash

# Kill all background processes started by this script upon exit
trap "kill 0" EXIT

echo "================================================"
echo "🚀 Starting Investment Portfolio Manager..."
echo "================================================"

# 1. Start Backend Rust
echo "⚙️  Starting Backend API on http://127.0.0.1:8000..."
cd backend_rust
/home/coder/.cargo/bin/cargo run &
BACKEND_PID=$!
cd ..

# Give backend a moment to spin up
sleep 1.5

# 2. Start Frontend Vite
echo "💻 Starting Frontend Web App on http://localhost:5173..."
cd frontend
npm run dev &
FRONTEND_PID=$!
cd ..

echo "================================================"
echo "✨ Both servers are running!"
echo "👉 Web App: http://localhost:5173"
echo "👉 API: http://127.0.0.1:8000"
echo "Press CTRL+C to stop both servers."
echo "================================================"

# Wait for background processes to finish (will block terminal)
wait
