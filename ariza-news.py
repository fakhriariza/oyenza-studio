#!/usr/bin/env python3
"""
Real-time News Fetcher for Ariza Studio.
Fetches breaking/trending news from CNBC Indonesia and Antara News.
Saves to /tmp/ariza_news.json
"""

import json
import re
import urllib.request

FEEDS = [
    ("CNBC", "https://www.cnbcindonesia.com/news/rss"),
    ("Antara", "https://www.antaranews.com/rss/terkini.xml"),
]

def fetch_feed(source, url):
    items = []
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"})
        with urllib.request.urlopen(req, timeout=3.5) as resp:
            content = resp.read().decode("utf-8", errors="ignore")
        
        matches = re.findall(r"<item>(.*?)</item>", content, re.DOTALL)
        for m in matches[:5]:
            title_match = re.search(r"<title>(.*?)</title>", m, re.DOTALL)
            link_match = re.search(r"<link>(.*?)</link>", m, re.DOTALL)
            if title_match:
                title = title_match.group(1).strip()
                title = re.sub(r"<!\[CDATA\[(.*?)\]\]>", r"\1", title).strip()
                title = re.sub(r"&amp;", "&", title)
                title = re.sub(r"&quot;", '"', title)
                link = link_match.group(1).strip() if link_match else ""
                items.append({
                    "source": source,
                    "title": title,
                    "link": link
                })
    except Exception:
        pass
    return items

def update():
    all_news = []
    for source, url in FEEDS:
        all_news.extend(fetch_feed(source, url))
    
    # Interleave / deduplicate
    if not all_news:
        return []
    
    # Save to /tmp/ariza_news.json
    try:
        with open("/tmp/ariza_news.json", "w", encoding="utf-8") as f:
            json.dump(all_news[:10], f, ensure_ascii=False, indent=2)
    except Exception:
        pass
    return all_news

if __name__ == "__main__":
    news = update()
    print(f"Fetched {len(news)} news items:")
    for n in news[:5]:
        print(f"[{n['source']}] {n['title']}")
