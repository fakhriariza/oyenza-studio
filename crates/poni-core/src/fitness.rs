//! Fitness, Intermittent Fasting, Weight Tracking & Food/Calorie Estimator for Ariza Studio.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FitnessStore {
    #[serde(default)]
    pub fasting: FastingState,
    #[serde(default)]
    pub weight_logs: Vec<WeightEntry>,
    #[serde(default)]
    pub target_weight: Option<f32>,
    #[serde(default)]
    pub initial_weight: Option<f32>,
    #[serde(default)]
    pub food_logs: BTreeMap<String, Vec<FoodEntry>>,
    #[serde(default = "default_calorie_target")]
    pub daily_calorie_target: u32,
    #[serde(default)]
    pub active_tab: u8, // 0: Fasting, 1: Weight, 2: Calorie & Food
}

fn default_calorie_target() -> u32 {
    2000
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FastingState {
    #[serde(default)]
    pub is_fasting: bool,
    #[serde(default)]
    pub fast_start_unix: Option<i64>,
    #[serde(default = "default_target_hours")]
    pub target_hours: u32,
    #[serde(default)]
    pub last_fast_completed: Option<i64>,
}

fn default_target_hours() -> u32 {
    16
}

impl Default for FastingState {
    fn default() -> Self {
        Self {
            is_fasting: false,
            fast_start_unix: None,
            target_hours: 16,
            last_fast_completed: None,
        }
    }
}

impl FastingState {
    pub fn start(&mut self, now_unix: i64) {
        self.is_fasting = true;
        self.fast_start_unix = Some(now_unix);
    }

    pub fn stop(&mut self, now_unix: i64) {
        self.is_fasting = false;
        self.last_fast_completed = Some(now_unix);
        self.fast_start_unix = None;
    }

    pub fn elapsed_secs(&self, now_unix: i64) -> u64 {
        if let Some(start) = self.fast_start_unix {
            if self.is_fasting && now_unix > start {
                return (now_unix - start) as u64;
            }
        }
        0
    }

    pub fn progress(&self, now_unix: i64) -> f32 {
        if !self.is_fasting || self.target_hours == 0 {
            return 0.0;
        }
        let elapsed = self.elapsed_secs(now_unix) as f32;
        let target = (self.target_hours * 3600) as f32;
        (elapsed / target).clamp(0.0, 2.0)
    }

    pub fn stage(&self, now_unix: i64) -> (&'static str, &'static str) {
        let hours = (self.elapsed_secs(now_unix) as f64) / 3600.0;
        if hours < 4.0 {
            ("Gula Darah Stabilizing", "Gula darah & insulin mulai turun ke batas basal")
        } else if hours < 8.0 {
            ("Saluran Cerna Istirahat", "Pencernaan istirahat, tubuh membersihkan sisa makanan")
        } else if hours < 12.0 {
            ("Pembakaran Lemak Aktif", "Glikogen hati menipis, lemak mulai dibakar (Gluconeogenesis)")
        } else if hours < 16.0 {
            ("Ketosis Dimulai", "Produksi keton meningkat, fokus meningkat & bakar lemak intensif")
        } else if hours < 24.0 {
            ("Autofagi & Cell Repair", "Sel-sel rusak didaur ulang dan regenerasi sel baru aktif")
        } else {
            ("Deep Ketosis & GH Boost", "Ketosis maksimal, hormon pertumbuhan (GH) meningkat tajam")
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeightEntry {
    pub timestamp: i64,
    pub date: String, // "YYYY-MM-DD"
    pub weight_kg: f32,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FoodEntry {
    pub id: u64,
    pub time: String, // "HH:MM"
    pub name: String,
    pub calories: u32,
    pub protein_g: f32,
    pub carbs_g: f32,
    pub fat_g: f32,
}

/// Estimates calories (kcal), protein (g), carbs (g), fat (g) from user input.
pub fn estimate_food(input: &str) -> (u32, f32, f32, f32) {
    let lower = input.trim().to_lowercase();
    if lower.is_empty() {
        return (0, 0.0, 0.0, 0.0);
    }

    // 1. Extract portion multiplier
    let portion = extract_portion(&lower);

    // 2. Base lookup dictionary (Indonesian & universal everyday foods)
    let base = match () {
        // Nasi & Karbo
        _ if lower.contains("nasi padang rendang") => (680, 32.0, 65.0, 34.0),
        _ if lower.contains("nasi padang ayam") || lower.contains("nasi padang") => (620, 28.0, 68.0, 26.0),
        _ if lower.contains("nasi goreng special") || lower.contains("nasi goreng telur") => (520, 16.0, 68.0, 20.0),
        _ if lower.contains("nasi goreng") => (450, 10.0, 65.0, 16.0),
        _ if lower.contains("nasi uduk") => (380, 8.0, 56.0, 14.0),
        _ if lower.contains("nasi kuning") => (360, 7.0, 54.0, 13.0),
        _ if lower.contains("nasi merah") => (180, 4.0, 38.0, 1.5),
        _ if lower.contains("nasi putih") || lower.contains("nasi") => (204, 4.2, 44.5, 0.4),
        _ if lower.contains("kentang rebus") => (130, 3.0, 30.0, 0.2),
        _ if lower.contains("kentang goreng") || lower.contains("french fries") => (365, 4.0, 48.0, 17.0),
        _ if lower.contains("oatmeal") || lower.contains("havermut") => (150, 5.0, 27.0, 2.5),
        _ if lower.contains("roti tawar") || lower.contains("roti gandum") => (140, 5.0, 26.0, 1.8),

        // Ayam & Daging
        _ if lower.contains("ayam geprek nasi") => (650, 28.0, 65.0, 32.0),
        _ if lower.contains("ayam geprek") => (420, 26.0, 18.0, 28.0),
        _ if lower.contains("dada ayam rebus") || lower.contains("dada ayam panggang") => (165, 31.0, 0.0, 3.6),
        _ if lower.contains("dada ayam") => (195, 30.0, 0.0, 7.5),
        _ if lower.contains("ayam goreng") || lower.contains("fried chicken") => (280, 22.0, 8.0, 18.0),
        _ if lower.contains("ayam bakar") => (220, 26.0, 4.0, 11.0),
        _ if lower.contains("rendang") => (340, 24.0, 6.0, 25.0),
        _ if lower.contains("sate ayam") => (380, 32.0, 16.0, 21.0),
        _ if lower.contains("sate kambing") => (430, 28.0, 12.0, 30.0),
        _ if lower.contains("soto ayam") => (320, 22.0, 24.0, 14.0),
        _ if lower.contains("soto betawi") || lower.contains("soto daging") => (450, 26.0, 18.0, 32.0),
        _ if lower.contains("rawon") => (380, 24.0, 16.0, 24.0),
        _ if lower.contains("bakso") => (380, 20.0, 32.0, 19.0),
        _ if lower.contains("mie ayam") => (450, 18.0, 62.0, 15.0),
        _ if lower.contains("steak sapi") || lower.contains("sirloin") || lower.contains("tenderloin") => (350, 38.0, 0.0, 22.0),

        // Telur, Tahu & Tempe
        _ if lower.contains("telur rebus") => (78, 6.3, 0.6, 5.3),
        _ if lower.contains("telur ceplok") || lower.contains("telur dadar") || lower.contains("telur mata sapi") => (110, 6.5, 0.8, 9.0),
        _ if lower.contains("telur") => (80, 6.5, 0.6, 5.5),
        _ if lower.contains("tempe goreng") => (118, 7.5, 7.0, 7.0),
        _ if lower.contains("tempe bacem") => (130, 8.0, 12.0, 5.0),
        _ if lower.contains("tempe") => (95, 9.5, 4.5, 5.0),
        _ if lower.contains("tahu goreng") => (90, 6.0, 4.0, 6.0),
        _ if lower.contains("tahu") => (70, 7.0, 2.0, 4.0),

        // Ikan & Seafood
        _ if lower.contains("ikan bakar") => (210, 28.0, 2.0, 10.0),
        _ if lower.contains("ikan goreng") || lower.contains("lele goreng") => (280, 24.0, 6.0, 18.0),
        _ if lower.contains("salmon") => (240, 25.0, 0.0, 15.0),
        _ if lower.contains("udang") => (120, 23.0, 1.0, 2.0),

        // Mie & Pasta
        _ if lower.contains("indomie goreng telur") || lower.contains("mie goreng telur") => (490, 15.0, 56.0, 23.0),
        _ if lower.contains("indomie goreng") || lower.contains("mie instan goreng") => (380, 8.0, 54.0, 15.0),
        _ if lower.contains("indomie kuah") || lower.contains("mie kuah") => (340, 7.0, 50.0, 13.0),
        _ if lower.contains("spaghetti") || lower.contains("pasta") => (380, 14.0, 64.0, 7.0),

        // Sayur & Salad
        _ if lower.contains("salad sayur") || lower.contains("salad") => (120, 3.0, 10.0, 7.0),
        _ if lower.contains("capcay") => (140, 6.0, 14.0, 6.0),
        _ if lower.contains("sayur sop") => (90, 3.0, 12.0, 2.5),
        _ if lower.contains("kangkung") || lower.contains("tumis kangkung") => (110, 3.5, 8.0, 7.0),
        _ if lower.contains("sayur bayam") => (65, 3.0, 8.0, 1.5),
        _ if lower.contains("gado-gado") || lower.contains("gado gado") => (380, 16.0, 36.0, 20.0),

        // Minuman & Kopi
        _ if lower.contains("kopi susu gula aren") || lower.contains("kopi kenangan") || lower.contains("kopi susu") => (190, 3.5, 28.0, 7.5),
        _ if lower.contains("kopi hitam") || lower.contains("espresso") || lower.contains("americano") => (5, 0.3, 0.5, 0.1),
        _ if lower.contains("latte") || lower.contains("cappuccino") => (140, 7.0, 12.0, 6.5),
        _ if lower.contains("boba") || lower.contains("milk tea") => (340, 4.0, 58.0, 11.0),
        _ if lower.contains("teh manis") => (90, 0.0, 22.0, 0.0),
        _ if lower.contains("teh tawar") || lower.contains("air putih") => (0, 0.0, 0.0, 0.0),
        _ if lower.contains("jus alpukat") => (260, 3.0, 32.0, 15.0),
        _ if lower.contains("jus buah") || lower.contains("jus jeruk") => (120, 1.5, 28.0, 0.5),

        // Buah-buahan
        _ if lower.contains("pisang") => (105, 1.3, 27.0, 0.3),
        _ if lower.contains("apel") => (95, 0.5, 25.0, 0.3),
        _ if lower.contains("alpukat") => (160, 2.0, 9.0, 15.0),
        _ if lower.contains("jeruk") => (62, 1.2, 15.0, 0.2),
        _ if lower.contains("mangga") => (135, 1.1, 35.0, 0.6),

        // Suplemen & Camilan
        _ if lower.contains("whey protein") || lower.contains("whey") => (120, 24.0, 2.5, 1.5),
        _ if lower.contains("susu uht") || lower.contains("susu full cream") => (150, 8.0, 12.0, 8.0),
        _ if lower.contains("protein bar") => (210, 20.0, 22.0, 6.0),
        _ if lower.contains("gorengan") || lower.contains("bakwan") => (140, 2.5, 16.0, 8.0),
        _ if lower.contains("martabak manis") => (380, 7.0, 52.0, 17.0),
        _ if lower.contains("martabak telur") => (320, 14.0, 22.0, 20.0),
        _ if lower.contains("pizza") => (285, 12.0, 34.0, 11.0),
        _ if lower.contains("burger") => (360, 18.0, 32.0, 18.0),

        // Fallback Heuristics
        _ if lower.contains("goreng") => (320, 10.0, 35.0, 16.0),
        _ if lower.contains("bakar") => (240, 18.0, 12.0, 12.0),
        _ if lower.contains("rebus") || lower.contains("kukus") => (140, 12.0, 14.0, 4.0),
        _ if lower.contains("manis") => (180, 2.0, 36.0, 4.0),
        _ => (250, 10.0, 30.0, 10.0),
    };

    let cal = ((base.0 as f32) * portion).round() as u32;
    let prot = (base.1 * portion * 10.0).round() / 10.0;
    let carb = (base.2 * portion * 10.0).round() / 10.0;
    let fat = (base.3 * portion * 10.0).round() / 10.0;

    (cal, prot, carb, fat)
}

fn extract_portion(s: &str) -> f32 {
    // Check words for numbers or specific quantity words
    let mut factor = 1.0f32;

    for word in s.split_whitespace() {
        // e.g. "2x", "3x", "2porsi", "100g", "200g", "2butir", "2potong"
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '.');
        if let Some(num_str) = clean.strip_suffix('x') {
            if let Ok(n) = num_str.parse::<f32>() {
                return n.clamp(0.1, 10.0);
            }
        }
        if let Some(num_str) = clean.strip_suffix('g') {
            if let Ok(n) = num_str.parse::<f32>() {
                return (n / 100.0).clamp(0.1, 15.0);
            }
        }
        if clean == "setengah" || clean == "1/2" {
            return 0.5;
        }
        if clean == "seperempat" || clean == "1/4" {
            return 0.25;
        }
        if clean == "double" || clean == "dua" {
            factor = 2.0;
        }
        if clean == "triple" || clean == "tiga" {
            factor = 3.0;
        }
        if let Ok(n) = clean.parse::<f32>() {
            if n > 0.0 && n <= 10.0 {
                factor = n;
            }
        }
    }
    factor
}
