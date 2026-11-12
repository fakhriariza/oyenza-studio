//! Belajar Fiqih & Kajian Islam Praktis Harian untuk Ariza Studio.
//! Mengikuti Apple / Poni Dynamic Island design system (palet warna asli, tipografi San Francisco, ikon vektor).

use crate::gfx::{TextStyle, Weight};
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid, wid_n, PillStyle, Ui};
use poni_core::geom::{palette, Rect};
use std::sync::{OnceLock, RwLock};

pub const HEIGHT: f32 = 256.0;

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

const REMOTE_FEED_URL: &str = "https://raw.githubusercontent.com/fakhriariza/oyenza-studio/main/data/fiqih_lessons.json";

pub fn lessons() -> Vec<FiqhLesson> {
    let cache = LESSONS_CACHE.get_or_init(|| {
        let local_path = crate::paths::data_dir().join("fiqih_lessons.json");
        if let Ok(content) = std::fs::read_to_string(&local_path) {
            if let Ok(parsed) = serde_json::from_str::<Vec<FiqhLesson>>(&content) {
                if !parsed.is_empty() {
                    return RwLock::new(parsed);
                }
            }
        }
        let fallback: Vec<FiqhLesson> = serde_json::from_str(EMBEDDED_JSON).unwrap_or_default();
        RwLock::new(fallback)
    });

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

fn sync_remote_lessons() {
    let Ok(client) = crate::services::http::Client::new("ArizaStudio") else { return; };
    let Ok(resp) = client.get(REMOTE_FEED_URL, std::time::Duration::from_secs(8)) else { return; };
    if resp.status == 200 {
        if let Ok(items) = serde_json::from_str::<Vec<FiqhLesson>>(&resp.body) {
            if !items.is_empty() {
                let data_dir = crate::paths::data_dir();
                let _ = std::fs::create_dir_all(&data_dir);
                let local_path = data_dir.join("fiqih_lessons.json");
                let _ = std::fs::write(&local_path, &resp.body);

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
    ("Fahruddin Faiz", "https://open.spotify.com/search/Fahruddin%20Faiz"),
    ("Hanan Attaki", "https://open.spotify.com/search/Hanan%20Attaki"),
    ("Adi Hidayat", "https://open.spotify.com/search/Adi%20Hidayat"),
    ("Buya Yahya", "https://open.spotify.com/search/Buya%20Yahya"),
    ("Murottal", "https://open.spotify.com/search/Murottal%20Mishary%20Rashid"),
    ("Dzikir", "https://open.spotify.com/search/Dzikir%20Pagi%20Petang%20Al%20Matsurat"),
];

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let accent = st.accent;
    let all_lessons = lessons();
    let n = all_lessons.len();
    if n == 0 {
        return;
    }

    let auto_slot = (st.unix / 7200) as usize;
    let current_idx = (auto_slot + st.ui.kajian_quote_idx) % n;
    let lesson = &all_lessons[current_idx];

    // 1. Header Row
    let header_y = area.y + 2.0;
    ui.c.text("Kajian Fiqih", Rect::new(area.x, header_y, 110.0, 20.0), &TextStyle::headline().color(palette::TEXT));
    ui.c.text("· berganti per 2 jam", Rect::new(area.x + 114.0, header_y + 3.0, 160.0, 16.0), &TextStyle::caption().color(palette::TEXT_FAINT));

    // Nav controls: Prev [ < ] Counter [ 1/30 ] Next [ > ]
    let btn_w = 26.0;
    let next_x = area.right() - btn_w;
    let count_w = 46.0;
    let count_x = next_x - count_w - 4.0;
    let prev_x = count_x - btn_w - 4.0;

    if ui.pill(wid("fiqh.prev"), Rect::new(prev_x, header_y - 2.0, btn_w, 24.0), "", Some(Icon::Prev), PillStyle::Utility).clicked {
        st.ui.kajian_quote_idx = (st.ui.kajian_quote_idx + n - 1) % n;
        st.ui.kajian_scroll = 0.0;
    }
    let counter_str = format!("{}/{}", current_idx + 1, n);
    ui.c.text(&counter_str, Rect::new(count_x, header_y + 2.0, count_w, 16.0), &TextStyle::footnote().tabular().color(palette::TEXT_DIM).center());
    if ui.pill(wid("fiqh.next"), Rect::new(next_x, header_y - 2.0, btn_w, 24.0), "", Some(Icon::Next), PillStyle::Utility).clicked {
        st.ui.kajian_quote_idx = (st.ui.kajian_quote_idx + 1) % n;
        st.ui.kajian_scroll = 0.0;
    }

    let is_expanded = st.ui.kajian_expanded;
    let card_y = header_y + 26.0;

    if !is_expanded {
        // ==================== MODE RINGKAS (COLLAPSED) ====================
        let card_h = 148.0;
        let card = Rect::new(area.x, card_y, area.w, card_h);
        ui.c.fill_rrect(card, 10.0, palette::SURFACE);
        ui.c.stroke_rrect(card, 10.0, palette::BORDER, 1.0);

        // Category Tag
        let cat_label = lesson.category.to_uppercase();
        ui.c.text(&cat_label, Rect::new(card.x + 14.0, card.y + 12.0, 200.0, 14.0), &TextStyle::caption().weight(Weight::Semibold).color(accent));

        // Read More button
        let readmore_w = 112.0;
        let readmore_rect = Rect::new(card.right() - readmore_w - 12.0, card.y + 8.0, readmore_w, 24.0);
        if ui.pill(wid("fiqh.readmore"), readmore_rect, "Baca Lengkap", None, PillStyle::Utility).clicked {
            st.ui.kajian_expanded = true;
            st.ui.kajian_scroll = 0.0;
        }

        // Title
        let title_st = TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT).wrap().top();
        let avail_title_w = card.w - readmore_w - 36.0;
        let (_, title_h) = ui.c.measure(&lesson.title, &title_st, avail_title_w);
        ui.c.text(&lesson.title, Rect::new(card.x + 14.0, card.y + 30.0, avail_title_w, title_h + 1.0), &title_st);

        // Summary
        let sum_y = card.y + 34.0 + title_h.max(20.0);
        let sum_st = TextStyle::footnote().color(palette::TEXT_DIM).wrap().top();
        let max_sum_h = (card.bottom() - 26.0 - sum_y).max(18.0);
        let (_, sum_h) = ui.c.measure(&lesson.summary, &sum_st, card.w - 28.0);
        let draw_sum_h = sum_h.min(max_sum_h);
        ui.c.text(&lesson.summary, Rect::new(card.x + 14.0, sum_y, card.w - 28.0, draw_sum_h + 1.0), &sum_st);

        // Dalil citation at bottom
        ui.c.text(&lesson.dalil, Rect::new(card.x + 14.0, card.bottom() - 20.0, card.w - 28.0, 14.0), &TextStyle::caption().color(palette::TEXT_FAINT));

        // Audio & Podcast row
        let audio_y = card.bottom() + 10.0;
        ui.c.text("Audio Kajian (Spotify)", Rect::new(area.x, audio_y, 200.0, 14.0), &TextStyle::caption().color(palette::TEXT_FAINT));

        let chips_y = audio_y + 16.0;
        let mut px = area.x;
        for (i, (label, url)) in PODCASTS.iter().enumerate() {
            let pw = ui.pill_width(label, true, true);
            if px + pw > area.right() {
                break;
            }
            let b = Rect::new(px, chips_y, pw, 24.0);
            if ui.pill(wid_n("kajian.podcast", i as u64), b, label, Some(Icon::Play), PillStyle::Utility).clicked {
                let _ = std::process::Command::new("open").arg(url).spawn();
            }
            px += pw + 6.0;
        }
    } else {
        // ==================== MODE DETAIL LENGKAP (EXPANDED READ MORE) ====================
        let card_h = area.bottom() - card_y;
        let card = Rect::new(area.x, card_y, area.w, card_h);
        ui.c.fill_rrect(card, 10.0, palette::SURFACE);
        ui.c.stroke_rrect(card, 10.0, palette::BORDER, 1.0);

        // Top action bar
        let cat_label = lesson.category.to_uppercase();
        ui.c.text(&cat_label, Rect::new(card.x + 14.0, card.y + 12.0, 180.0, 14.0), &TextStyle::caption().weight(Weight::Semibold).color(accent));

        let close_w = 72.0;
        let close_r = Rect::new(card.right() - close_w - 12.0, card.y + 8.0, close_w, 24.0);
        if ui.pill(wid("fiqh.collapse"), close_r, "Tutup", Some(Icon::Close), PillStyle::Utility).clicked {
            st.ui.kajian_expanded = false;
        }

        let copy_w = 68.0;
        let copy_r = Rect::new(close_r.x - copy_w - 6.0, card.y + 8.0, copy_w, 24.0);
        if ui.pill(wid("fiqh.copy"), copy_r, "Salin", Some(Icon::Copy), PillStyle::Utility).clicked {
            let copy_text = format!(
                "{}\n[{}]\n\nIntisari:\n{}\n\nPanduan Praktis:\n{}\n\nDalil:\n{}\n\n(Ariza Studio)",
                lesson.title, lesson.category, lesson.summary, lesson.detail, lesson.dalil
            );
            #[cfg(target_os = "macos")]
            crate::services::clipboard::set_text(&copy_text);
        }

        let view = Rect::new(card.x + 14.0, card.y + 36.0, card.w - 28.0, card.h - 44.0);

        let title_st = TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT).wrap().top();
        let (_, title_h) = ui.c.measure(&lesson.title, &title_st, view.w);

        let sum_st = TextStyle::footnote().color(palette::TEXT).wrap().top();
        let (_, sum_h) = ui.c.measure(&lesson.summary, &sum_st, view.w);

        let det_st = TextStyle::footnote().color(palette::TEXT_DIM).wrap().top();
        let (_, det_h) = ui.c.measure(&lesson.detail, &det_st, view.w);

        let dalil_st = TextStyle::footnote().color(palette::TEXT_FAINT).wrap().top();
        let (_, dalil_h) = ui.c.measure(&lesson.dalil, &dalil_st, view.w);

        let total_content_h = title_h + 8.0 + 16.0 + sum_h + 12.0 + 16.0 + det_h + 12.0 + 16.0 + dalil_h + 16.0;
        let max_scroll = (total_content_h - view.h).max(0.0);
        if max_scroll > 0.0 {
            let notches = ui.wheel_in(card);
            st.ui.kajian_scroll = (st.ui.kajian_scroll - notches * 26.0).clamp(0.0, max_scroll);
        }
        let scroll = st.ui.kajian_scroll;

        ui.c.clip(view);

        let mut y = view.y - scroll;

        // Title
        ui.c.text(&lesson.title, Rect::new(view.x, y, view.w, title_h + 1.0), &title_st);
        y += title_h + 10.0;

        // Section 1: Intisari
        ui.c.text("INTISARI", Rect::new(view.x, y, view.w, 14.0), &TextStyle::caption().weight(Weight::Semibold).color(accent));
        y += 16.0;
        ui.c.text(&lesson.summary, Rect::new(view.x, y, view.w, sum_h + 1.0), &sum_st);
        y += sum_h + 12.0;

        // Section 2: Panduan Praktis
        ui.c.text("PANDUAN PRAKTIS", Rect::new(view.x, y, view.w, 14.0), &TextStyle::caption().weight(Weight::Semibold).color(accent));
        y += 16.0;
        ui.c.text(&lesson.detail, Rect::new(view.x, y, view.w, det_h + 1.0), &det_st);
        y += det_h + 12.0;

        // Section 3: Dalil & Rujukan
        ui.c.text("DALIL & RUJUKAN", Rect::new(view.x, y, view.w, 14.0), &TextStyle::caption().weight(Weight::Semibold).color(palette::TEXT_FAINT));
        y += 16.0;
        ui.c.text(&lesson.dalil, Rect::new(view.x, y, view.w, dalil_h + 1.0), &dalil_st);

        ui.c.unclip();
    }
}
