#!/usr/bin/env python3
"""
Spotify Web API integration for Ariza Studio.
Supports OAuth 2.0 PKCE, listing user playlists & tracks, and controlling playback directly from the bar.
"""

import base64
import hashlib
import http.server
import json
import os
import secrets
import sys
import urllib.parse
import urllib.request
import webbrowser

CONFIG_DIR = os.path.expanduser("~/.config/ariza-studio")
CONFIG_FILE = os.path.join(CONFIG_DIR, "spotify.json")
CACHE_FILE = "/tmp/ariza_spotify_playlists.json"
REDIRECT_URI = "http://127.0.0.1:8888/callback"
SCOPES = "user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private user-top-read user-read-recently-played"

def load_config():
    if os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, "r") as f:
                return json.load(f)
        except Exception:
            pass
    return {}

def save_config(cfg):
    os.makedirs(CONFIG_DIR, exist_ok=True)
    with open(CONFIG_FILE, "w") as f:
        json.dump(cfg, f, indent=2)

def generate_pkce_codes():
    verifier = secrets.token_urlsafe(64)[:128]
    digest = hashlib.sha256(verifier.encode("ascii")).digest()
    challenge = base64.urlsafe_b64encode(digest).decode("ascii").rstrip("=")
    return verifier, challenge

def refresh_access_token():
    cfg = load_config()
    client_id = cfg.get("client_id")
    refresh_token = cfg.get("refresh_token")
    if not client_id or not refresh_token:
        return None
    
    data = urllib.parse.urlencode({
        "grant_type": "refresh_token",
        "refresh_token": refresh_token,
        "client_id": client_id,
    }).encode("utf-8")

    req = urllib.request.Request(
        "https://accounts.spotify.com/api/token",
        data=data,
        headers={"Content-Type": "application/x-www-form-urlencoded"}
    )
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            tokens = json.loads(resp.read().decode("utf-8"))
            cfg["access_token"] = tokens.get("access_token")
            if "refresh_token" in tokens:
                cfg["refresh_token"] = tokens.get("refresh_token")
            save_config(cfg)
            return cfg["access_token"]
    except Exception as e:
        print(f"Error refreshing Spotify token: {e}", file=sys.stderr)
        return None

def get_valid_token():
    cfg = load_config()
    token = cfg.get("access_token")
    if not token and cfg.get("refresh_token"):
        token = refresh_access_token()
    return token

def api_request(method, endpoint, body=None):
    token = get_valid_token()
    if not token:
        return {"error": "not_authenticated"}
    
    url = f"https://api.spotify.com/v1/{endpoint.lstrip('/')}"
    headers = {"Authorization": f"Bearer {token}", "Content-Type": "application/json"}
    data = json.dumps(body).encode("utf-8") if body else None

    req = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            if resp.status == 204:
                return {"ok": True}
            raw = resp.read().decode("utf-8")
            return json.loads(raw) if raw else {"ok": True}
    except urllib.error.HTTPError as e:
        if e.code == 401:
            # Token expired, try once to refresh
            new_token = refresh_access_token()
            if new_token:
                headers["Authorization"] = f"Bearer {new_token}"
                req = urllib.request.Request(url, data=data, headers=headers, method=method)
                try:
                    with urllib.request.urlopen(req, timeout=5) as resp2:
                        if resp2.status == 204:
                            return {"ok": True}
                        raw = resp2.read().decode("utf-8")
                        return json.loads(raw) if raw else {"ok": True}
                except Exception as ex2:
                    return {"error": str(ex2)}
        return {"error": f"HTTP {e.code}: {e.reason}"}
    except Exception as e:
        return {"error": str(e)}

def do_login(client_id=None):
    cfg = load_config()
    if client_id:
        cfg["client_id"] = client_id
        save_config(cfg)
    else:
        client_id = cfg.get("client_id")
        
    if not client_id:
        print("Masukkan Spotify Client ID kamu terlebih dahulu:")
        print("  ariza-spotify setup <CLIENT_ID>")
        print("(Bisa didapat gratis di https://developer.spotify.com/dashboard)")
        sys.exit(1)

    verifier, challenge = generate_pkce_codes()
    state = secrets.token_hex(16)

    params = {
        "client_id": client_id,
        "response_type": "code",
        "redirect_uri": REDIRECT_URI,
        "code_challenge_method": "S256",
        "code_challenge": challenge,
        "state": state,
        "scope": SCOPES,
    }
    auth_url = f"https://accounts.spotify.com/authorize?{urllib.parse.urlencode(params)}"

    auth_code = None

    class CallbackHandler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            nonlocal auth_code
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == "/callback":
                qs = urllib.parse.parse_qs(parsed.query)
                if "code" in qs:
                    auth_code = qs["code"][0]
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(b"<h1>Login Berhasil!</h1><p>Spotify sudah terhubung ke Ariza Studio. Kamu bisa tutup halaman ini.</p>")
                else:
                    self.send_response(400)
                    self.end_headers()
                    self.wfile.write(b"Gagal mendapatkan authorization code.")
            else:
                self.send_response(404)
                self.end_headers()
        def log_message(self, format, *args):
            return

    server = http.server.HTTPServer(("127.0.0.1", 8888), CallbackHandler)
    server.timeout = 60

    print("Membuka browser untuk login ke Spotify...")
    webbrowser.open(auth_url)

    server.handle_request()
    server.server_close()

    if not auth_code:
        print("Login gagal atau timeout (tidak ada kode yang diterima).")
        sys.exit(1)

    # Tukar authorization code dengan tokens
    data = urllib.parse.urlencode({
        "grant_type": "authorization_code",
        "code": auth_code,
        "redirect_uri": REDIRECT_URI,
        "client_id": client_id,
        "code_verifier": verifier,
    }).encode("utf-8")

    req = urllib.request.Request(
        "https://accounts.spotify.com/api/token",
        data=data,
        headers={"Content-Type": "application/x-www-form-urlencoded"}
    )
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            tokens = json.loads(resp.read().decode("utf-8"))
            cfg["access_token"] = tokens.get("access_token")
            cfg["refresh_token"] = tokens.get("refresh_token")
            save_config(cfg)
            print("✓ Spotify berhasil terhubung ke Ariza Studio!")
            fetch_playlists()
    except Exception as e:
        print(f"Gagal menukar token: {e}")
        sys.exit(1)

def fetch_playlists():
    res = api_request("GET", "me/playlists?limit=20")
    if "items" in res:
        playlists = []
        for item in res["items"]:
            playlists.append({
                "name": item.get("name"),
                "uri": item.get("uri"),
                "tracks_total": item.get("tracks", {}).get("total", 0),
                "image": item.get("images", [{}])[0].get("url") if item.get("images") else None
            })
        try:
            with open(CACHE_FILE, "w") as f:
                json.dump(playlists, f, indent=2)
            print(f"✓ Tersimpan {len(playlists)} playlist ke cache.")
        except Exception:
            pass
        return playlists
    return []

def play_uri(uri):
    body = {}
    if "track" in uri:
        body["uris"] = [uri]
    else:
        body["context_uri"] = uri
    res = api_request("PUT", "me/player/play", body)
    if "error" in res:
        print(f"Gagal memutar: {res['error']}")
        return False
    print(f"✓ Memutar {uri} di Spotify!")
    return True

def pause():
    api_request("PUT", "me/player/pause")

def next_track():
    api_request("POST", "me/player/next")

def prev_track():
    api_request("POST", "me/player/previous")

def status():
    res = api_request("GET", "me/player")
    if "item" in res and res["item"]:
        track = res["item"].get("name")
        artists = ", ".join([a.get("name") for a in res["item"].get("artists", [])])
        is_playing = res.get("is_playing", False)
        print(f"{'▶' if is_playing else '⏸'} {track} - {artists}")
    else:
        print("Tidak ada playback aktif di Spotify.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  ariza-spotify setup <client_id>")
        print("  ariza-spotify login")
        print("  ariza-spotify playlists")
        print("  ariza-spotify play <spotify_uri>")
        print("  ariza-spotify pause")
        print("  ariza-spotify next")
        print("  ariza-spotify prev")
        print("  ariza-spotify status")
        sys.exit(0)

    cmd = sys.argv[1].lower()
    if cmd == "setup":
        if len(sys.argv) < 3:
            print("Usage: ariza-spotify setup <client_id>")
            sys.exit(1)
        cfg = load_config()
        cfg["client_id"] = sys.argv[2].strip()
        save_config(cfg)
        print(f"✓ Spotify Client ID tersimpan: {cfg['client_id']}")
    elif cmd == "login":
        cid = sys.argv[2] if len(sys.argv) > 2 else None
        do_login(cid)
    elif cmd in ("playlists", "list"):
        plist = fetch_playlists()
        for i, p in enumerate(plist, 1):
            print(f"{i}. {p['name']} ({p['tracks_total']} lagu) -> {p['uri']}")
    elif cmd == "play":
        if len(sys.argv) < 3:
            print("Usage: ariza-spotify play <uri>")
            sys.exit(1)
        play_uri(sys.argv[2])
    elif cmd == "pause":
        pause()
    elif cmd == "next":
        next_track()
    elif cmd in ("prev", "previous"):
        prev_track()
    elif cmd == "status":
        status()
    else:
        print(f"Perintah tidak dikenal: {cmd}")
