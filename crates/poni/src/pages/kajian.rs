//! Belajar Fiqih & Kajian Islam Praktis Harian untuk Ariza Studio.
//! Mengganti materi otomatis setiap beberapa jam (Fiqih Sholat, Thaharah, Puasa, Muamalah, Adab).
//! Mendukung auto-sync dari GitHub Raw JSON dan offline caching ke disk.

use crate::gfx::{TextStyle, Weight};
use crate::state::AppState;
use crate::ui::{wid, wid_n, PillStyle, Ui};
use poni_core::geom::{palette, Color, Rect};
use std::sync::{OnceLock, RwLock};

pub const HEIGHT: f32 = 224.0;

pub fn height(_st: &AppState) -> f32 {
    HEIGHT
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FiqhLesson {
    pub category: String,
    pub title: String,
    pub summary: String,
    pub detail: String,
    pub dalil: String,
}

static LESSONS_CACHE: OnceLock<RwLock<Vec<FiqhLesson>>> = OnceLock::new();
static SYNC_TRIGGERED: OnceLock<()> = OnceLock::new();
const EMBEDDED_JSON: &str = include_str!("../../../../data/fiqih_lessons.json");

/// URL remote feed di repo GitHub
const REMOTE_FEED_URL: &str = "https://raw.githubusercontent.com/fakhriariza/oyenza-studio/main/data/fiqih_lessons.json";

/// Mengambil daftar materi fiqih (dari memory cache / file disk lokal / embedded fallback).
pub fn lessons() -> Vec<FiqhLesson> {
    let cache = LESSONS_CACHE.get_or_init(|| {
        // 1. Coba baca dari file lokal disk di data_dir()
        let local_path = crate::paths::data_dir().join("fiqih_lessons.json");
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            if let Ok(parsed) = serde_json::from_str::<Vec<FiqhLesson>>(&content) {
                if !parsed.is_empty() {
                    return RwLock::new(parsed);
                }
            }
        }
        // 2. Fallback ke embedded JSON
        let fallback: Vec<FiqhLesson> = serde_json::from_str(EMBEDDED_JSON).unwrap_or_default();
        RwLock::new(fallback)
    });

    // Jalankan sync online di background thread secara non-blocking
    SYNC_TRIGGERED.get_or_init(|| {
        std::thread::Builder::new()
            .name("poni-fiqih-sync".into())
            .spawn(|| {
                sync_remote_lessons();
            })
            .ok();
    });

    if let Ok(guard) = cache.read() {
        guard.clone()
    } else {
        serde_json::from_str(EMBEDDED_JSON).unwrap_or_default()
    }
}

/// Sinkronisasi feed materi fiqih terbaru dari GitHub Raw saat online.
fn sync_remote_lessons() {
    let Ok(client) = crate::services::http::Client::new("ArizaStudio") else { return; };
    let Ok(resp) = client.get(REMOTE_FEED_URL, std::time::Duration::from_secs(8)) else { return; };
    if resp.status == 200 {
        if let Ok(items) = serde_json::from_str::<Vec<FiqhLesson>>(&resp.body) {
            if !items.is_empty() {
                // Simpan ke local cache di disk
                let data_dir = crate::paths::data_dir();
                let _ = std::fs::create_dir_all(&data_dir);
                let local_path = data_dir.join("fiqih_lessons.json");
                let _ = std::fs::write(&local_path, &resp.body);

                // Perbarui cache di memori
                if let Some(lock) = LESSONS_CACHE.get() {
                    if let Ok(mut guard) = lock.write() {
                        *guard = items;
                    }
                }
            }
        }
    }
}

const PODCASTS: &[(&str, &str)] = &[
    ("🎙️ Fahruddin Faiz", "https://open.spotify.com/search/Fahruddin%20Faiz"),
    ("🎙️ Hanan Attaki", "https://open.spotify.com/search/Hanan%20Attaki"),
    ("🎙️ Adi Hidayat", "https://open.spotify.com/search/Adi%20Hidayat"),
    ("🎙️ Buya Yahya", "https://open.spotify.com/search/Buya%20Yahya"),
    ("📖 Murottal 30 Juz", "https://open.spotify.com/search/Murottal%20Mishary%20Rashid"),
    ("🌅 Dzikir Pagi Petang", "https://open.spotify.com/search/Dzikir%20Pagi%20Petang%20Al%20Matsurat"),
];

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let accent = st.accent;
    let all_lessons = lessons();
    let n = all_lessons.len();
    if n == 0 {
        return;
    }

    // Auto-rotasi setiap 2 jam (7200 detik), atau digeser manual oleh tombol navigasi
    let auto_slot = (st.unix / 7200) as usize;
    let current_idx = (auto_slot + st.ui.kajian_quote_idx) % n;
    let lesson = &all_lessons[current_idx];

    // 1. Header Bar: Title + Navigation buttons
    let header_y = area.y + 2.0;
    ui.c.text("Belajar Fiqih & Kajian Praktis 📚", Rect::new(area.x, header_y, 250.0, 20.0), &TextStyle::headline().weight(Weight::Bold).color(accent));

    // Nav buttons: Prev, Counter, Next
    let counter_str = format!("{}/{}", current_idx + 1, n);
    let nav_btn_w = 28.0;
    let next_x = area.right() - nav_btn_w;
    let count_w = 44.0;
    let count_x = next_x - count_w - 4.0;
    let prev_x = count_x - nav_btn_w - 4.0;

    if ui.pill(wid("fiqh.prev"), Rect::new(prev_x, header_y - 2.0, nav_btn_w, 24.0), "◀", None, PillStyle::Utility).clicked {
        st.ui.kajian_quote_idx = (st.ui.kajian_quote_idx + n - 1) % n;
    }
    ui.c.text(&counter_str, Rect::new(count_x, header_y + 1.0, count_w, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));
    if ui.pill(wid("fiqh.next"), Rect::new(next_x, header_y - 2.0, nav_btn_w, 24.0), "▶", None, PillStyle::Utility).clicked {
        st.ui.kajian_quote_idx = (st.ui.kajian_quote_idx + 1) % n;
    }

    // 2. Fiqh Study Card
    let card_y = header_y + 24.0;
    let card_h = 114.0;
    let card = Rect::new(area.x, card_y, area.w, card_h);
    ui.c.fill_rrect(card, 8.0, Color::white(0.06));

    // Category badge & Title
    let cat_badge = format!("[ {} ]", lesson.category);
    ui.c.text(&cat_badge, Rect::new(card.x + 12.0, card.y + 8.0, 140.0, 16.0), &TextStyle::footnote().weight(Weight::Bold).color(palette::GREEN));
    ui.c.text(&lesson.title, Rect::new(card.x + 12.0, card.y + 26.0, card.w - 24.0, 20.0), &TextStyle::body().weight(Weight::Bold));

    // Summary (Intisari)
    ui.c.text(&lesson.summary, Rect::new(card.x + 12.0, card.y + 48.0, card.w - 24.0, 18.0), &TextStyle::footnote());

    // Detail & Dalil
    ui.c.text(&lesson.detail, Rect::new(card.x + 12.0, card.y + 68.0, card.w - 24.0, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));
    let dalil_str = format!("📌 Dalil: {}", lesson.dalil);
    ui.c.text(&dalil_str, Rect::new(card.x + 12.0, card.y + 90.0, card.w - 24.0, 16.0), &TextStyle::caption().color(palette::ORANGE));

    // 3. Audio & Podcast Shortcut row (Buka di Spotify)
    let audio_y = card.bottom() + 10.0;
    ui.c.text("Putar Audio Kajian & Murottal (Spotify):", Rect::new(area.x, audio_y, 250.0, 16.0), &TextStyle::caption().color(palette::TEXT_DIM));

    let chips_y = audio_y + 16.0;
    let mut px = area.x;
    for (i, (label, url)) in PODCASTS.iter().enumerate() {
        let pw = ui.pill_width(label, false, true);
        if px + pw > area.right() {
            break;
        }
        let b = Rect::new(px, chips_y, pw, 24.0);
        if ui.pill(wid_n("kajian.podcast", i as u64), b, label, None, PillStyle::Primary(palette::GREEN)).clicked {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
        px += pw + 6.0;
    }
}
