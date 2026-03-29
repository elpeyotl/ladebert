#!/bin/bash
# Build, sign, package DMG, and create GitHub release
set -e

echo "🔨 Building Ladebert..."
source "$HOME/.cargo/env"
npm run tauri build

APP_PATH="src-tauri/target/release/bundle/macos/Ladebert.app"
DMG_DIR="src-tauri/target/release/bundle/dmg"

echo ""
echo "🔏 Signing app (ad-hoc)..."
codesign --force --deep --sign - "$APP_PATH"
echo "   Signatur OK: $(codesign -dv "$APP_PATH" 2>&1 | grep 'Signature')"

echo ""
echo "📦 Re-creating DMG with signed app..."
DMG_OUT="$DMG_DIR/Ladebert.dmg"
rm -f "$DMG_OUT"
hdiutil create -volname "Ladebert" \
  -srcfolder "$APP_PATH" \
  -ov -format UDZO \
  "$DMG_OUT"

cp "$DMG_OUT" "landingpage/Ladebert.dmg"

# Get version from package.json
VERSION=$(node -p "require('./package.json').version")

echo ""
echo "🚀 Creating GitHub Release v${VERSION}..."
if gh release view "v${VERSION}" &>/dev/null; then
  echo "   Release v${VERSION} exists, updating..."
  gh release upload "v${VERSION}" "landingpage/Ladebert.dmg" --clobber
else
  gh release create "v${VERSION}" \
    "landingpage/Ladebert.dmg" \
    --title "Ladebert v${VERSION}" \
    --notes-file CHANGELOG.md
fi

echo ""
echo "✅ Done!"
echo "   Release: https://github.com/elpeyotl/ladebert/releases/tag/v${VERSION}"
