#!/usr/bin/env python3
"""
Spotify Web API integration for Ariza Studio.
Supports interactive OAuth 2.0 PKCE with setup wizard, listing playlists & tracks,
and controlling playback directly from the notch bar.
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
PORT = 8888
REDIRECT_URI = f"http://127.0.0.1:{PORT}/callback"
SCOPES = "user-read-playback-state user-modify-playback-state user-read-currently-playing playlist-read-private user-top-read user-read-recently-played"
DEFAULT_CLIENT_ID = "17d4ad02c14447449b9d89e3c1e68e86"

HTML_SETUP = """<!DOCTYPE html>
<html lang="id">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Connect Spotify · Ariza Studio</title>
<style>
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body {
    background: #121212;
    color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 24px;
  }
  .card {
    background: #1e1e1e;
    border: 1px solid #2a2a2a;
    border-radius: 16px;
    max-width: 500px;
    width: 100%;
    padding: 32px;
    box-shadow: 0 16px 32px rgba(0,0,0,0.4);
  }
  .header { display: flex; align-items: center; gap: 12px; margin-bottom: 24px; }
  .logo {
    width: 44px; height: 44px; border-radius: 50%;
    background: #1db954; display: flex; align-items: center; justify-content: center;
    font-size: 24px; font-weight: bold; color: black;
  }
  h1 { font-size: 22px; font-weight: 700; }
  p.subtitle { color: #a0a0a0; font-size: 14px; margin-top: 2px; }
  .steps {
    background: #161616;
    border: 1px solid #282828;
    border-radius: 12px;
    padding: 16px 20px;
    margin-bottom: 24px;
    font-size: 13.5px;
    line-height: 1.6;
    color: #d0d0d0;
  }
  .steps ol { padding-left: 20px; }
  .steps li { margin-bottom: 8px; }
  .steps a { color: #1ed760; text-decoration: none; font-weight: 600; }
  .steps a:hover { text-decoration: underline; }
  .code-badge {
    background: #2a2a2a;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: ui-monospace, Menlo, monospace;
    font-size: 12px;
    color: #1ed760;
  }
  label { display: block; font-size: 13px; font-weight: 600; margin-bottom: 8px; color: #e0e0e0; }
  input[type="text"] {
    width: 100%;
    background: #282828;
    border: 1px solid #3e3e3e;
    border-radius: 8px;
    color: white;
    padding: 12px 14px;
    font-size: 14px;
    font-family: ui-monospace, monospace;
    margin-bottom: 20px;
    outline: none;
  }
  input[type="text"]:focus { border-color: #1db954; }
  button {
    width: 100%;
    background: #1db954;
    color: black;
    border: none;
    border-radius: 8px;
    padding: 14px;
    font-size: 15px;
    font-weight: 700;
    cursor: pointer;
    transition: background 0.2s;
  }
  button:hover { background: #1ed760; }
</style>
</head>
<body>
<div class="card">
  <div class="header">
    <div class="logo">♫</div>
    <div>
      <h1>Connect Spotify</h1>
      <p class="subtitle">Ariza Studio Web API Integration</p>
    </div>
  </div>

  <div class="steps">
    <ol>
      <li>Buka <a href="https://developer.spotify.com/dashboard" target="_blank">Spotify Developer Dashboard ↗</a> lalu login.</li>
      <li>Klik <b>Create App</b>:
        <br>• App name: <span class="code-badge">Ariza Studio</span>
        <br>• Redirect URI: <span class="code-badge">http://127.0.0.1:8888/callback</span>
        <br>• APIs used: centang <b>Web API</b></li>
      <li>Copy <b>Client ID</b> kamu dan paste di bawah ini:</li>
    </ol>
  </div>

  <form method="POST" action="/save_client">
    <label for="cid">Spotify Client ID</label>
    <input type="text" id="cid" name="client_id" placeholder="misal: 7a8b9c0d1e2f3a4b..." required autofocus>
    <button type="submit">Lanjut Login Spotify ➔</button>
  </form>
</div>
</body>
</html>
"""

HTML_SUCCESS = """<!DOCTYPE html>
<html lang="id">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Login Berhasil · Ariza Studio</title>
<style>
  body {
    background: #121212; color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    display: flex; align-items: center; justify-content: center; min-height: 100vh;
  }
  .card {
    background: #1e1e1e; border: 1px solid #282828; border-radius: 16px;
    max-width: 440px; padding: 40px; text-align: center;
    box-shadow: 0 16px 32px rgba(0,0,0,0.4);
  }
  .icon { font-size: 54px; margin-bottom: 16px; }
  h1 { font-size: 22px; font-weight: 700; margin-bottom: 8px; color: #1ed760; }
  p { color: #a0a0a0; font-size: 14px; line-height: 1.5; }
</style>
</head>
<body>
<div class="card">
  <div class="icon">🎉</div>
  <h1>Spotify Berhasil Terhubung!</h1>
  <p>Playlist Spotify kamu sudah tersinkronisasi ke <b>Ariza Studio</b> di Dynamic Island. Kamu bisa menutup tab browser ini.</p>
</div>
</body>
</html>
"""

def load_config():
    cfg = {"client_id": DEFAULT_CLIENT_ID}
    if os.path.exists(CONFIG_FILE):
        try:
            with open(CONFIG_FILE, "r") as f:
                loaded = json.load(f)
                cfg.update(loaded)
        except Exception:
            pass
    return cfg

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

def do_oauth_flow(client_id):
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
    return f"https://accounts.spotify.com/authorize?{urllib.parse.urlencode(params)}", verifier

def run_server_and_login():
    cfg = load_config()
    client_id = cfg.get("client_id")

    verifier = None
    auth_code = None

    class ServerHandler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            nonlocal auth_code, verifier
            parsed = urllib.parse.urlparse(self.path)
            
            if parsed.path == "/":
                if not client_id:
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(HTML_SETUP.encode("utf-8"))
                else:
                    auth_url, v = do_oauth_flow(client_id)
                    verifier = v
                    self.send_response(302)
                    self.send_header("Location", auth_url)
                    self.end_headers()
            elif parsed.path == "/callback":
                qs = urllib.parse.parse_qs(parsed.query)
                if "code" in qs:
                    auth_code = qs["code"][0]
                    self.send_response(200)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(HTML_SUCCESS.encode("utf-8"))
                else:
                    self.send_response(400)
                    self.send_header("Content-Type", "text/html; charset=utf-8")
                    self.end_headers()
                    self.wfile.write(b"<h1>Gagal: Tidak ada authorization code</h1>")
            else:
                self.send_response(404)
                self.end_headers()

        def do_POST(self):
            nonlocal client_id, verifier
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == "/save_client":
                length = int(self.headers.get("content-length", 0))
                body = self.rfile.read(length).decode("utf-8")
                qs = urllib.parse.parse_qs(body)
                cid = qs.get("client_id", [""])[0].strip()
                if cid:
                    client_id = cid
                    cfg["client_id"] = cid
                    save_config(cfg)
                    auth_url, v = do_oauth_flow(cid)
                    verifier = v
                    self.send_response(302)
                    self.send_header("Location", auth_url)
                    self.end_headers()
                    return
            self.send_response(400)
            self.end_headers()

        def log_message(self, format, *args):
            return

    server = http.server.ThreadingHTTPServer(("127.0.0.1", PORT), ServerHandler)
    server.timeout = 180

    print(f"Membuka browser untuk Spotify setup di http://127.0.0.1:{PORT}/ ...")
    webbrowser.open(f"http://127.0.0.1:{PORT}/")

    # Handle request until callback received or timeout
    while not auth_code:
        server.handle_request()

    server.server_close()

    if not auth_code or not verifier:
        print("Login gagal atau dibatalkan.")
        return

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
            # Send notification to Dynamic Island
            os.system("/Users/dikshatek/Downloads/oyenza-studio/ariza-notify --title 'Spotify Connected' --done 'Spotify Web API berhasil terhubung!'")
    except Exception as e:
        print(f"Gagal menukar token: {e}")

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
    if len(sys.argv) < 2 or sys.argv[1] == "login":
        run_server_and_login()
    else:
        cmd = sys.argv[1].lower()
        if cmd == "setup":
            if len(sys.argv) < 3:
                print("Usage: ariza-spotify setup <client_id>")
                sys.exit(1)
            cfg = load_config()
            cfg["client_id"] = sys.argv[2].strip()
            save_config(cfg)
            print(f"✓ Spotify Client ID tersimpan: {cfg['client_id']}")
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
