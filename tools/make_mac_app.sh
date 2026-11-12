#!/bin/bash
# Build "Oyenza Studio.app" and its DMG from Poni's Mac binaries. Runs on a Mac (lipo, codesign, hdiutil).
#
#   tools/make_mac_app.sh <arm64 binary> <x86_64 binary | -> <OyenzaStudio.icns | -> <out dir>
#
# The app is ad-hoc signed (no Developer ID yet), so the DMG is a test build: macOS asks once, in
# System Settings > Privacy & Security > Open Anyway, before it first opens. The DMG's window says so
# (tools/make_mac_dmg.sh). tools/sign_mac_dmg.sh makes the signed, notarized DMG from this one.
set -euo pipefail

arm="$1"; intel="$2"; icns="$3"; out="$4"
root="$(cd "$(dirname "$0")/.." && pwd)"
version="$(cargo metadata --no-deps --format-version 1 --manifest-path "$root/Cargo.toml" \
  | python3 -c 'import sys, json; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"] == "poni"))')"

mkdir -p "$out"
app="$out/Ariza Studio.app"
rm -rf "$app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

# One binary for Apple Silicon and Intel Macs.
if [ "$intel" != "-" ]; then
  lipo -create -output "$app/Contents/MacOS/poni" "$arm" "$intel"
else
  cp "$arm" "$app/Contents/MacOS/poni"
fi
chmod +x "$app/Contents/MacOS/poni"
lipo -info "$app/Contents/MacOS/poni"

sed "s/@VERSION@/$version/g" "$root/installer/mac/Info.plist" > "$app/Contents/Info.plist"
plutil -lint "$app/Contents/Info.plist"
printf 'APPL????' > "$app/Contents/PkgInfo"
if [ "$icns" != "-" ]; then
  # Named the way Info.plist asks for it, so a renamed build (ArizaStudio) still finds its icon.
  icon_name="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIconFile' "$app/Contents/Info.plist" 2>/dev/null || true)"
  cp "$icns" "$app/Contents/Resources/${icon_name:-ArizaStudio}.icns"
fi

# MediaRemote adapter framework & scripts
mkdir -p "$app/Contents/Frameworks"
if [ -d "$root/Frameworks/MediaRemoteAdapter.framework" ]; then
  cp -R "$root/Frameworks/MediaRemoteAdapter.framework" "$app/Contents/Frameworks/"
elif [ -n "${MEDIAREMOTE_BUILD:-}" ]; then
  cp -R "$MEDIAREMOTE_BUILD/MediaRemoteAdapter.framework" "$app/Contents/Frameworks/"
fi

if [ -f "$root/third_party/mediaremote-adapter/bin/mediaremote-adapter.pl" ]; then
  cp "$root/third_party/mediaremote-adapter/bin/mediaremote-adapter.pl" "$app/Contents/Resources/"
fi

# Copy Ariza helper scripts
for helper in ariza-spotify.py ariza-notify ariza-news.py; do
  if [ -f "$root/$helper" ]; then
    cp "$root/$helper" "$app/Contents/Resources/"
    chmod +x "$app/Contents/Resources/$helper"
  fi
done

# Ad-hoc signature: seals the bundle, the adapter's framework too
codesign --force --deep --sign - "$app"
codesign --verify --deep --strict --verbose=2 "$app"

echo "built $app"
