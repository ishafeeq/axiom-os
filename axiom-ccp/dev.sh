#!/bin/bash

# Configuration
BACKEND_DIR="axiom-ccp-backend"
FRONTEND_DIR="axiom-ccp-frontend"
BACKEND_PORT=3000
FRONTEND_PORT=5173
LOG_DIR=".logs"

# Ensure log directory exists
mkdir -p $LOG_DIR

echo "🚀 Starting Axiom CCP Stack..."

# 1. Kill existing processes on target ports
echo "🧹 Clearing ports $BACKEND_PORT and $FRONTEND_PORT..."
lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null
lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null

# 2. Cleanup function for graceful shutdown
cleanup() {
    echo -e "\n🛑 Shutting down Axiom CCP..."
    kill $BACKEND_PID $FRONTEND_PID 2>/dev/null
    exit
}

trap cleanup SIGINT

# 3. Start Backend
echo "📡 Launching Backend..."
cd $BACKEND_DIR
export TMPDIR=/tmp && export CARGO_HOME=/tmp/cargo_cache
cargo run --target-dir /tmp/axiom_target > "../$LOG_DIR/backend.log" 2>&1 &
BACKEND_PID=$!
cd ..

# 4. Start Frontend
echo "💻 Launching Frontend..."
cd $FRONTEND_DIR
npm run dev > "../$LOG_DIR/frontend.log" 2>&1 &
FRONTEND_PID=$!
cd ..

echo "✅ Services started in background."
echo "📜 Tailing logs (Ctrl+C to stop everything)..."
echo "-------------------------------------------"

# 5. Tail Logs
tail -f "$LOG_DIR/backend.log" "$LOG_DIR/frontend.log"
