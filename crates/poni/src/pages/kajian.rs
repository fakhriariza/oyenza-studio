//! Kajian, Podcast, Murottal & Islamic Wisdom page for Ariza Studio.

use crate::gfx::{TextStyle, Weight};
use crate::state::AppState;
use crate::ui::{wid, wid_n, PillStyle, Ui};
use poni_core::geom::{palette, Color, Rect};

pub const HEIGHT: f32 = 216.0;

pub fn height(_st: &AppState) -> f32 {
    HEIGHT
}

const QUOTES: &[(&str, &str)] = &[
    (
        "\"Jangan menunggu bahagia baru bersyukur, tapi bersyukurlah maka kamu akan bahagia.\"",
        "— Dr. Fahruddin Faiz (Ngaji Filsafat)",
    ),
    (
        "\"Dunia ini persinggahan sementara, jangan sampai yang sementara membuatmu lupa yang abadi.\"",
        "— Nasehat Ulama",
    ),
    (
        "\"Sebaik-baik manusia adalah yang paling bermanfaat bagi orang lain.\"",
        "— HR. Ahmad",
    ),
    (
        "\"Ketenangan hati tidak ditemukan dalam banyaknya harta, melainkan dalam lapangnya rasa syukur.\"",
        "— Buya Yahya",
    ),
    (
        "\"Ikhlas itu seperti surat Al-Ikhlas: tidak ada kata ikhlas di dalamnya, tapi maknanya penuh kemurnian.\"",
        "— Hikmah Islam",
    ),
    (
        "\"Barangsiapa mempermudah urusan orang lain, Allah akan mempermudah urusannya di dunia dan akhirat.\"",
        "— HR. Muslim",
    ),
    (
        "\"Hati yang tenang adalah buah dari dzikir dan mengingat Allah dalam setiap helaan nafas.\"",
        "— Ust. Adi Hidayat",
    ),
    (
        "\"Jangan pernah lelah berbuat baik, sekecil apapun itu, karena kamu tidak tahu kebaikan mana yang mengantarkanmu ke surga.\"",
        "— Ust. Hanan Attaki",
    ),
];

const PODCASTS: &[(&str, &str)] = &[
    ("🎙️ Fahruddin Faiz", "https://open.spotify.com/search/Fahruddin%20Faiz"),
    ("🎙️ Hanan Attaki", "https://open.spotify.com/search/Hanan%20Attaki"),
    ("🎙️ Adi Hidayat", "https://open.spotify.com/search/Adi%20Hidayat"),
    ("🎙️ Buya Yahya", "https://open.spotify.com/search/Buya%20Yahya"),
    ("📖 Murottal 30 Juz", "https://open.spotify.com/search/Murottal%20Mishary%20Rashid"),
    ("🌅 Dzikir Pagi Petang", "https://open.spotify.com/search/Dzikir%20Pagi%20Petang%20Al%20Matsurat"),
];

const LIVE_STREAMS: &[(&str, &str)] = &[
    ("🕋 Live Makkah Haramain", "https://www.youtube.com/results?search_query=live+makkah+haramain+24+jam"),
    ("🕌 Live Masjid Nabawi", "https://www.youtube.com/results?search_query=live+masjid+nabawi+madinah"),
    ("📻 Radio Rodja 756 AM", "https://radiorodja.com/live/"),
];

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let accent = st.accent;

    // 1. Header title
    let header_y = area.y + 2.0;
    ui.c.text("Kajian & Penyejuk Hati 🎙️", Rect::new(area.x, header_y, 220.0, 20.0), &TextStyle::headline().weight(Weight::Bold).color(accent));
    ui.c.text("Ngaji Filsafat, Podcast, Murottal & Dzikir Harian", Rect::new(area.x + 224.0, header_y + 2.0, area.w - 224.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

    // 2. Daily Wisdom Quote Card
    let quote_idx = st.ui.kajian_quote_idx % QUOTES.len();
    let (quote_text, quote_author) = QUOTES[quote_idx];

    let card_y = header_y + 24.0;
    let card_h = 52.0;
    let card = Rect::new(area.x, card_y, area.w, card_h);
    ui.c.fill_rrect(card, 8.0, Color::white(0.06));

    ui.c.text(quote_text, Rect::new(card.x + 12.0, card.y + 8.0, card.w - 110.0, 20.0), &TextStyle::body().weight(Weight::Medium));
    ui.c.text(quote_author, Rect::new(card.x + 12.0, card.y + 28.0, card.w - 110.0, 16.0), &TextStyle::footnote().color(palette::GREEN));

    // Button to rotate quote
    let next_btn_r = Rect::new(card.right() - 94.0, card.center().y - 12.0, 84.0, 24.0);
    if ui.pill(wid("kajian.next_quote"), next_btn_r, "Ganti Quote", None, PillStyle::Utility).clicked {
        st.ui.kajian_quote_idx = (st.ui.kajian_quote_idx + 1) % QUOTES.len();
    }

    // 3. Podcast & Kajian Audio Chips (1-Click Play via Spotify/Browser)
    let p_title_y = card.bottom() + 10.0;
    ui.c.text("Pilihan Kajian & Podcast (Buka di Spotify):", Rect::new(area.x, p_title_y, 300.0, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));

    let chips_y = p_title_y + 18.0;
    let mut px = area.x;
    for (i, (label, url)) in PODCASTS.iter().enumerate() {
        let pw = ui.pill_width(label, false, true);
        if px + pw > area.right() {
            break;
        }
        let b = Rect::new(px, chips_y, pw, 26.0);
        if ui.pill(wid_n("kajian.podcast", i as u64), b, label, None, PillStyle::Primary(palette::GREEN)).clicked {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
        px += pw + 8.0;
    }

    // 4. Live Streaming & Siaran Langsung (Makkah, Madinah, Radio Rodja)
    let live_title_y = chips_y + 34.0;
    ui.c.text("Siaran Langsung & Live Stream:", Rect::new(area.x, live_title_y, 250.0, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));

    let live_y = live_title_y + 18.0;
    let mut lx = area.x;
    for (i, (label, url)) in LIVE_STREAMS.iter().enumerate() {
        let pw = ui.pill_width(label, false, true);
        if lx + pw > area.right() {
            break;
        }
        let b = Rect::new(lx, live_y, pw, 24.0);
        if ui.pill(wid_n("kajian.live", i as u64), b, label, None, PillStyle::Utility).clicked {
            let _ = std::process::Command::new("open").arg(url).spawn();
        }
        lx += pw + 8.0;
    }
}
