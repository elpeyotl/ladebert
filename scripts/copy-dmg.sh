#!/bin/bash
# Copies the latest DMG from Tauri build output to the landingpage folder

DMG_DIR="src-tauri/target/release/bundle/dmg"
DEST="landingpage"

DMG_FILE=$(ls -t "$DMG_DIR"/*.dmg 2>/dev/null | head -1)

if [ -z "$DMG_FILE" ]; then
  echo "No DMG found in $DMG_DIR"
  exit 1
fi

FILENAME=$(basename "$DMG_FILE")
cp "$DMG_FILE" "$DEST/$FILENAME"

# Also create a generic latest link name
cp "$DMG_FILE" "$DEST/Ladebert.dmg"

echo "Copied $FILENAME -> $DEST/Ladebert.dmg"
