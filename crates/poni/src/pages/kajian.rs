//! Belajar Fiqih & Kajian Islam Praktis Harian untuk Ariza Studio.
//! Mengganti materi otomatis setiap beberapa jam (Fiqih Sholat, Thaharah, Puasa, Muamalah, Adab).

use crate::gfx::{TextStyle, Weight};
use crate::state::AppState;
use crate::ui::{wid, wid_n, PillStyle, Ui};
use poni_core::geom::{palette, Color, Rect};

pub const HEIGHT: f32 = 224.0;

pub fn height(_st: &AppState) -> f32 {
    HEIGHT
}

pub struct FiqhLesson {
    pub category: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub detail: &'static str,
    pub dalil: &'static str,
}

pub const LESSONS: &[FiqhLesson] = &[
    FiqhLesson {
        category: "Fiqih Sholat",
        title: "Aturan Makmum Masbuq (Ketinggalan Rakaat)",
        summary: "Jika mendapati imam sedang ruku' dan sempat thuma'ninah bersama imam sebelum imam i'tidal, rakaat tersebut sah terhitung.",
        detail: "Takbiratul ihram sambil berdiri tegak dahulu, baru takbir intiqal ruku'. Lanjutkan kekurangan rakaat setelah salam imam.",
        dalil: "HR. Abu Dawud No. 893 & HR. Bukhari",
    },
    FiqhLesson {
        category: "Fiqih Sholat",
        title: "Sujud Sahwi: Kapan Sebelum vs Sesudah Salam?",
        summary: "Sebelum salam jika kurang rukun/sunnah ab'adh (lupa tasyahud awal) atau ragu jumlah rakaat dan ambil jumlah terkecil.",
        detail: "Sesudah salam jika kelebihan rakaat atau ragu tapi yakin pada jumlah yang lebih kuat. Sujud 2 kali lalu salam lagi.",
        dalil: "HR. Bukhari No. 1224 & Muslim No. 570",
    },
    FiqhLesson {
        category: "Fiqih Sholat",
        title: "Syarat & Batas Jamak Qashar saat Safar",
        summary: "Jarak safar minimal ±80-88 km dan sudah keluar batas kota/pemukiman domisili.",
        detail: "Dzuhur-Ashar & Maghrib-Isya boleh dijamak. Qashar hanya untuk sholat 4 rakaat menjadi 2 rakaat (Subuh & Maghrib tidak diqashar).",
        dalil: "QS. An-Nisa: 101 & HR. Muslim No. 686",
    },
    FiqhLesson {
        category: "Thaharah",
        title: "Mengusap Kaus Kaki saat Wudhu (Al-Mash 'ala Al-Khuffain)",
        summary: "Boleh mengusap bagian atas kaus kaki basah saat wudhu jika kaus kaki dipakai dalam keadaan suci (sudah wudhu sebelumnya).",
        detail: "Berlaku 1x24 jam bagi yang mukim (di rumah/kantor) dan 3x24 jam bagi musafir, selama hanya hadats kecil.",
        dalil: "HR. Muslim No. 276 dari Ali bin Abi Thalib",
    },
    FiqhLesson {
        category: "Thaharah",
        title: "Keraguan Batal Wudhu (Was-was Buang Angin)",
        summary: "Jangan batalkan sholat atau wudhu hanya karena merasa ragu di perut kecuali mendengar suara atau mencium baunya.",
        detail: "Darah luka kecil, mimisan, dan muntah sedikit tidak membatalkan wudhu menurut jumhur ulama.",
        dalil: "HR. Bukhari No. 137 & Muslim No. 361",
    },
    FiqhLesson {
        category: "Fiqih Sholat",
        title: "Thuma'ninah adalah Rukun Wajib Sholat",
        summary: "Thuma'ninah artinya diam sejenak setelah seluruh persendian tenang (minimal seukuran membaca 'Subhanallah').",
        detail: "Wajib di 4 tempat: Ruku', I'tidal, Sujud, dan Duduk di antara dua sujud. Sholat tanpa thuma'ninah tidak sah.",
        dalil: "Kisah orang yang buruk sholatnya (HR. Bukhari No. 757)",
    },
    FiqhLesson {
        category: "Fiqih Puasa",
        title: "Tindakan Medis Modern yang Tidak Membatalkan Puasa",
        summary: "Inhaler asma, obat tetes mata/telinga, suntikan insulin/vitamin otot, dan donor darah tidak membatalkan puasa.",
        detail: "Yang membatalkan adalah infus glukosa/nutrisi makanan yang menggantikan fungsi makan/minum ke pembuluh darah.",
        dalil: "Fatwa Majma' Al-Fiqh Al-Islami OKI No. 93",
    },
    FiqhLesson {
        category: "Dzikir & Sunnah",
        title: "Urutan Dzikir Ba'da Sholat Fardhu Sesuai Sunnah",
        summary: "Istighfar 3x ➔ Allahumma antas salam ➔ Ayat Kursi ➔ Tasbih, Tahmid, Takbir (33x) ➔ Tahlil penutup (1x).",
        detail: "Keutamaan Ayat Kursi ba'da sholat: Tiada yang menghalanginya masuk surga melainkan kematian.",
        dalil: "HR. An-Nasa'i & dishahihkan Ibnu Hibban",
    },
    FiqhLesson {
        category: "Muamalah",
        title: "Kaedah Emas Muamalah: Asal Jual Beli adalah Halal",
        summary: "Hukum asal muamalah halal kecuali ada: Riba, Gharar (ketidakjelasan fatal), Penipuan/Tadlis, dan Objek Haram.",
        detail: "Bunga pinjaman uang (apapun namanya: biaya admin harian, denda telat) adalah riba yang disepakati keharamannya.",
        dalil: "QS. Al-Baqarah: 275 & Kaedah Fiqhiyyah",
    },
    FiqhLesson {
        category: "Fiqih Sholat",
        title: "Sunnah Memasang Sutrah (Pembatas) Sholat",
        summary: "Dianjurkan meletakkan pembatas (dinding, tiang, tas, atau sajadah) di depan tempat sujud.",
        detail: "Fungsinya mencegah orang lewat persis di depan sujud dan menjaga pandangan mata tetap fokus.",
        dalil: "HR. Abu Dawud No. 698",
    },
    FiqhLesson {
        category: "Thaharah",
        title: "Tata Cara Tayamum Praktis yang Sah",
        summary: "Niat ➔ Tepuk debu bersih 1x ➔ Tiup debu berlebih ➔ Usap wajah sekali ➔ Usap kedua telapak tangan.",
        detail: "Cukup sampai pergelangan tangan (bukan sampai siku menurut hadits Ammar bin Yasir yang paling shahih).",
        dalil: "HR. Bukhari No. 338 & Muslim No. 368",
    },
    FiqhLesson {
        category: "Dzikir & Doa",
        title: "Waktu-Waktu Paling Mustajab untuk Berdoa",
        summary: "1) Sepertiga malam terakhir, 2) Antara adzan & iqamah, 3) Saat sujud, 4) Saat turun hujan, 5) 1 jam terakhir ba'da Ashar di hari Jum'at.",
        detail: "Awali dengan memuji Allah (Alhamdulillah) & shalawat Nabi, akui dosa, lalu panjatkan hajat dengan penuh keyakinan.",
        dalil: "HR. Tirmidzi No. 212 & HR. Abu Dawud",
    },
];

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
    let n = LESSONS.len();

    // Auto-rotasi setiap 2 jam (7200 detik), atau digeser manual oleh tombol navigasi
    let auto_slot = (st.unix / 7200) as usize;
    let current_idx = (auto_slot + st.ui.kajian_quote_idx) % n;
    let lesson = &LESSONS[current_idx];

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
    ui.c.text(lesson.title, Rect::new(card.x + 12.0, card.y + 26.0, card.w - 24.0, 20.0), &TextStyle::body().weight(Weight::Bold));

    // Summary (Intisari)
    ui.c.text(lesson.summary, Rect::new(card.x + 12.0, card.y + 48.0, card.w - 24.0, 18.0), &TextStyle::footnote());

    // Detail & Dalil
    ui.c.text(lesson.detail, Rect::new(card.x + 12.0, card.y + 68.0, card.w - 24.0, 18.0), &TextStyle::footnote().color(palette::TEXT_DIM));
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
