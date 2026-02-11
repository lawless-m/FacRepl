#!/bin/bash
# Start Factorio server with RCON + client

RCON_PASSWORD="${FACTORIO_RCON_PASSWORD:-yourpassword}"
RCON_PORT="${FACTORIO_RCON_PORT:-27015}"
FACTORIO_BIN=~/Factorio/game/bin/x64/factorio

echo "=========================================="
echo "FacRepl - Starting Factorio Server + Client"
echo "=========================================="
echo "RCON Port: $RCON_PORT"
echo "RCON Password: $RCON_PASSWORD"
echo ""

# Start the server in the background
echo "Starting Factorio server..."
if [ -n "$FACTORIO_SAVE" ]; then
    echo "Using save file: $FACTORIO_SAVE"
    $FACTORIO_BIN \
      --start-server "$FACTORIO_SAVE" \
      --rcon-port "$RCON_PORT" \
      --rcon-password "$RCON_PASSWORD" \
      &
else
    echo "Using most recent save file"
    $FACTORIO_BIN \
      --start-server-load-latest \
      --rcon-port "$RCON_PORT" \
      --rcon-password "$RCON_PASSWORD" \
      &
fi

SERVER_PID=$!
echo "Server started (PID: $SERVER_PID)"
echo ""

# Wait for server to initialize
echo "Waiting for server to start (10 seconds)..."
sleep 10

# Check if server is still running
if ! ps -p $SERVER_PID > /dev/null; then
    echo "ERROR: Server failed to start!"
    exit 1
fi

echo "Server should be ready!"
echo ""
echo "Starting Factorio client..."
echo "Connect to: localhost"
echo ""
echo "Press Ctrl+C to stop both server and client"
echo "=========================================="
echo ""

# Start the client
$FACTORIO_BIN &
CLIENT_PID=$!

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Shutting down..."
    kill $CLIENT_PID 2>/dev/null
    kill $SERVER_PID 2>/dev/null
    wait $CLIENT_PID 2>/dev/null
    wait $SERVER_PID 2>/dev/null
    echo "Done!"
}

trap cleanup EXIT INT TERM

# Wait for client to exit
wait $CLIENT_PID

# When client exits, kill the server
kill $SERVER_PID 2>/dev/null
wait $SERVER_PID 2>/dev/null

echo "Client exited. Server stopped."
