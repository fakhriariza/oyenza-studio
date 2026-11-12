#!/usr/bin/env bash
# Ariza Studio - 1-Click Universal Installer for macOS
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/fakhriariza/oyenza-studio/main/installer/install.sh | bash

set -e

echo ""
echo "=========================================="
echo "    🚀 Memasang Ariza Studio untuk Mac    "
echo "=========================================="
echo ""

RELEASE_URL="https://github.com/fakhriariza/oyenza-studio/releases/download/v0.2.30/Ariza-Studio-macOS.zip"
TMP_DIR="/tmp/ariza_install_$$"
mkdir -p "$TMP_DIR"
trap 'rm -rf "$TMP_DIR"' EXIT

# 1. Download zip dari GitHub Release
echo "📥 Mengunduh Ariza Studio (~2.5 MB)..."
curl -fSL "$RELEASE_URL" -o "$TMP_DIR/Ariza-Studio.zip"

# 2. Extract
echo "📦 Mengekstrak paket..."
unzip -q "$TMP_DIR/Ariza-Studio.zip" -d "$TMP_DIR"

# 3. Pindahkan ke /Applications
echo "📲 Memindahkan ke /Applications..."
pkill -9 poni 2>/dev/null || true
rm -rf "/Applications/Ariza Studio.app"
cp -R "$TMP_DIR/Ariza Studio.app" "/Applications/"

# 4. Hapus macOS Gatekeeper Quarantine (Fix 'App is damaged')
echo "🛡️  Membuka izin keamanan macOS (Gatekeeper)..."
xattr -cr "/Applications/Ariza Studio.app"

# 5. Pasang CLI ariza-notify & ariza-spotify
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"
if [ -d "$TMP_DIR/Ariza Studio.app/Contents/Resources" ]; then
  RES="$TMP_DIR/Ariza Studio.app/Contents/Resources"
  cp "$RES/ariza-notify" "$BIN_DIR/ariza-notify" 2>/dev/null || true
  cp "$RES/ariza-spotify.py" "$BIN_DIR/ariza-spotify" 2>/dev/null || true
  chmod +x "$BIN_DIR/ariza-notify" "$BIN_DIR/ariza-spotify" 2>/dev/null || true
fi

# Ensure ~/.local/bin in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
fi

# 6. Setup Antigravity Hooks
mkdir -p "$HOME/.gemini/config"
cat << 'EOF' > "$HOME/.gemini/config/hooks.json"
{
  "ariza-notifier": {
    "Stop": [
      {
        "type": "command",
        "command": "ariza-notify --title 'Antigravity' --done 'Task selesai / AI udah beres!'"
      }
    ],
    "PreToolUse": [
      {
        "matcher": "ask_question",
        "hooks": [
          {
            "type": "command",
            "command": "ariza-notify --title 'Antigravity' --acc",
            "timeout": 45
          }
        ]
      }
    ]
  }
}
EOF

# 7. Setup Claude Code Hooks
mkdir -p "$HOME/.claude"
cat << 'EOF' > "$HOME/.claude/settings.json"
{
  "hooks": {
    "Stop": [
      {
        "type": "command",
        "command": "ariza-notify --title 'Claude Code' --done 'Task selesai / Claude Code beres!'"
      }
    ],
    "Notification": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "ariza-notify --title 'Claude Code' --acc 'Claude Code butuh approval kamu'"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "type": "command",
        "command": "ariza-notify --title 'Claude Code' --running"
      }
    ]
  }
}
EOF

# 8. Buka Ariza Studio
echo "✨ Menjalankan Ariza Studio..."
open "/Applications/Ariza Studio.app"

echo ""
echo "=========================================="
echo "  🎉 SUKSES! Ariza Studio Siap Digunakan  "
echo "=========================================="
echo "Fitur aktif:"
echo " • Dynamic Island / Notch Interactive Answering"
echo " • Jadwal Sholat Offline + Notifikasi H-1 Jam"
echo " • Spotify & Browser Music Controller + Synced Lyrics"
echo " • Notifikasi Otomatis Antigravity, Claude Code & DeepSeek"
echo " • Berita Real-time CNBC & Antara saat AI berjalan"
echo "=========================================="
echo ""
