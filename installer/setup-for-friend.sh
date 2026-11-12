#!/usr/bin/env bash
# Ariza Studio - 1-Click Installer for Friends
# Installs Ariza Studio, terminal CLIs, and hooks for DeepSeek, Claude Code, and Antigravity.

set -e

echo "🚀 Installing Ariza Studio..."

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

# 1. Install CLI tools
echo "📦 Installing ariza-notify and ariza-spotify..."
cp "$ROOT/ariza-notify" "$BIN_DIR/ariza-notify"
cp "$ROOT/ariza-spotify.py" "$BIN_DIR/ariza-spotify"
chmod +x "$BIN_DIR/ariza-notify" "$BIN_DIR/ariza-spotify"

# Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
  echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$HOME/.zshrc"
  export PATH="$HOME/.local/bin:$PATH"
fi

# 2. Setup Antigravity Hooks
GEMINI_DIR="$HOME/.gemini/config"
mkdir -p "$GEMINI_DIR"
if [ ! -f "$GEMINI_DIR/hooks.json" ]; then
  cat << 'EOF' > "$GEMINI_DIR/hooks.json"
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
            "command": "ariza-notify --title 'Antigravity' --acc 'Butuh konfirmasi / jawaban kamu'"
          }
        ]
      }
    ]
  }
}
EOF
  echo "✓ Antigravity hooks configured"
fi

# 3. Setup Claude Code Hooks
CLAUDE_DIR="$HOME/.claude"
mkdir -p "$CLAUDE_DIR"
cat << 'EOF' > "$CLAUDE_DIR/settings.json"
{
  "hooks": {
    "Stop": [
      {
        "type": "command",
        "command": "ariza-notify --title 'Claude Code' --done 'Task selesai / Claude Code udah beres!'"
      }
    ],
    "Notification": [
      {
        "matcher": ".*",
        "hooks": [
          {
            "type": "command",
            "command": "ariza-notify --title 'Claude Code' --acc 'Claude Code butuh approval / perhatian kamu (acc)'"
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
echo "✓ Claude Code hooks configured"

# 4. Copy app to /Applications if bundle exists
if [ -d "$ROOT/build/Ariza Studio.app" ]; then
  echo "📲 Copying Ariza Studio to /Applications..."
  cp -R "$ROOT/build/Ariza Studio.app" "/Applications/"
  xattr -dr com.apple.quarantine "/Applications/Ariza Studio.app" 2>/dev/null || true
  open "/Applications/Ariza Studio.app"
  echo "🎉 Ariza Studio is running!"
fi

echo "✨ Setup complete! Ariza Studio is ready to use."
