#!/bin/bash
# Start Factorio as a server with RCON enabled for FacRepl

RCON_PASSWORD="${FACTORIO_RCON_PASSWORD:-yourpassword}"
RCON_PORT="${FACTORIO_RCON_PORT:-27015}"
SERVER_CONFIG_DIR="$HOME/.factorio-server"

echo "Starting Factorio server with RCON enabled..."
echo "RCON Port: $RCON_PORT"
echo "RCON Password: $RCON_PASSWORD"
echo "Server config: $SERVER_CONFIG_DIR"
echo ""

# Create server config directory and copy mod
mkdir -p "$SERVER_CONFIG_DIR/mods"
cp -r ~/Factorio/game/mods/factorio-constraint-builder_0.1.0 "$SERVER_CONFIG_DIR/mods/" 2>/dev/null || true

# Copy saves to server config if they don't exist
if [ ! -d "$SERVER_CONFIG_DIR/saves" ]; then
    mkdir -p "$SERVER_CONFIG_DIR/saves"
    cp ~/Factorio/saves/*.zip "$SERVER_CONFIG_DIR/saves/" 2>/dev/null || true
fi

# Create basic config if it doesn't exist
if [ ! -f "$SERVER_CONFIG_DIR/config.ini" ]; then
    cat > "$SERVER_CONFIG_DIR/config.ini" << 'EOF'
[path]
read-data=__PATH__executable__/../../data
write-data=__PATH__system-write-data__

[other]
check_updates=false
EOF
fi

# Check if a specific save file is provided
if [ -n "$FACTORIO_SAVE" ]; then
    echo "Using save file: $FACTORIO_SAVE"
    ~/Factorio/game/bin/x64/factorio \
      --config "$SERVER_CONFIG_DIR/config.ini" \
      --start-server "$FACTORIO_SAVE" \
      --rcon-port "$RCON_PORT" \
      --rcon-password "$RCON_PASSWORD"
else
    echo "Using most recent save file (--start-server-load-latest)"
    echo ""
    echo "To use a specific save instead, run:"
    echo "  export FACTORIO_SAVE=\"$SERVER_CONFIG_DIR/saves/your-save.zip\""
    echo "  ./start-factorio-rcon.sh"
    echo ""
    ~/Factorio/game/bin/x64/factorio \
      --config "$SERVER_CONFIG_DIR/config.ini" \
      --start-server-load-latest \
      --rcon-port "$RCON_PORT" \
      --rcon-password "$RCON_PASSWORD"
fi
