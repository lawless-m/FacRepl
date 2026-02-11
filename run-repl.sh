#!/bin/bash
# Run the FacRepl REPL

RCON_PASSWORD="${FACTORIO_RCON_PASSWORD:-yourpassword}"
RCON_PORT="${FACTORIO_RCON_PORT:-27015}"
RCON_HOST="${FACTORIO_RCON_HOST:-localhost}"

./rust-tools/target/release/fcb-repl \
  --host "$RCON_HOST" \
  --port "$RCON_PORT" \
  --password "$RCON_PASSWORD"
