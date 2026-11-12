#!/usr/bin/env python3
"""
DeepSeek Harness session watcher for Ariza Studio.
Automatically monitors ~/.dsh/sessions and triggers Dynamic Island notifications:
- 'Needs Approval (acc)' when an approval/question is requested.
- 'Finished' when a turn/task is completed.
"""

import glob
import os
import subprocess
import time

SESSIONS_DIR = os.path.expanduser("~/.dsh/sessions")
NOTIFY_CMD = os.path.expanduser("~/.local/bin/ariza-notify")

seen_events = set()
file_mtimes = {}

def get_latest_session_file():
    pattern = os.path.join(SESSIONS_DIR, "**", "session.v4.jsonl.zstd")
    files = glob.glob(pattern, recursive=True)
    if not files:
        return None
    return max(files, key=os.path.getmtime)

def inspect_file(filepath):
    try:
        res = subprocess.run(
            ["zstd", "-dc", filepath],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            timeout=2,
        )
        if res.returncode != 0 or not res.stdout:
            return

        lines = res.stdout.strip().splitlines()
        recent = lines[-5:] if len(lines) >= 5 else lines

        for line in recent:
            if not line:
                continue
            line_hash = hash(line)
            if line_hash in seen_events:
                continue
            seen_events.add(line_hash)

            if "ask_user_question" in line or "approval/request" in line:
                print("Detected approval request from DeepSeek Harness!")
                subprocess.run([NOTIFY_CMD, "--title", "DeepSeek Harness", "--acc", "Butuh approval / konfirmasi kamu (acc)"])
            elif '"type":"turn/end"' in line or '"type":"response/completed"' in line:
                print("Detected turn completion from DeepSeek Harness!")
                subprocess.run([NOTIFY_CMD, "--title", "DeepSeek Harness", "--done", "Task selesai / AI udah beres!"])
    except Exception as e:
        pass

def main():
    print(f"Ariza Studio - DeepSeek Harness watcher active on {SESSIONS_DIR}")
    # Prime current files so we don't spam past events on launch
    latest = get_latest_session_file()
    if latest:
        file_mtimes[latest] = os.path.getmtime(latest)
        try:
            res = subprocess.run(["zstd", "-dc", latest], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, timeout=2)
            if res.stdout:
                for line in res.stdout.strip().splitlines():
                    seen_events.add(hash(line))
        except Exception:
            pass

    while True:
        try:
            target = get_latest_session_file()
            if target:
                mtime = os.path.getmtime(target)
                prev_mtime = file_mtimes.get(target, 0)
                if mtime > prev_mtime:
                    file_mtimes[target] = mtime
                    inspect_file(target)
        except Exception:
            pass
        time.sleep(1.0)

if __name__ == "__main__":
    main()
