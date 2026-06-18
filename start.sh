#!/bin/bash

# Kill all background processes started by this script upon exit
trap "kill 0" EXIT

echo "================================================"
echo "🚀 Starting Investment Portfolio Manager..."
echo "================================================"

# 1. Start Backend FastAPI
echo "⚙️  Starting Backend API on http://127.0.0.1:8000..."
source backend/.venv/bin/activate
cd backend
uvicorn app.main:app --host 127.0.0.1 --port 8000 &
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
echo "👉 API Docs: http://127.0.0.1:8000/docs"
echo "Press CTRL+C to stop both servers."
echo "================================================"

# Wait for background processes to finish (will block terminal)
wait
