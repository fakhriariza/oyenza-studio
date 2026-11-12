//! Fitness, Intermittent Fasting, Weight Tracker & Food Calorie Estimator page for Ariza Studio.
//! Mengikuti Apple / Poni Dynamic Island design system (palet warna asli, tipografi San Francisco, ikon vektor).

use crate::gfx::{TextStyle, Weight};
use crate::icons::Icon;
use crate::pages::PageKind;
use crate::state::{AppState, Notice, NoticeAction};
use crate::ui::{wid, wid_n, PillStyle, Ui, PILL_SMALL};
use poni_core::datetime::{fmt_clock, fmt_date_short, LocalDateTime};
use poni_core::fitness::{estimate_food, FoodEntry, WeightEntry};
use poni_core::geom::{palette, Color, Rect};
use poni_core::health::fmt_steps;

pub const HEIGHT: f32 = 216.0;

pub fn height(_st: &AppState) -> f32 {
    HEIGHT
}

pub fn goal_notice(steps: u32, until: f64) -> Notice {
    Notice {
        icon: Icon::Steps,
        color: palette::GREEN,
        title: format!("{} steps today", fmt_steps(steps)),
        title_dim: String::new(),
        subtitle: "Daily step goal reached".into(),
        until,
        action: Some(NoticeAction::OpenPage(PageKind::Health)),
        inline: None,
        reveal: None,
    }
}

pub(crate) fn updated_text(st: &AppState, at: i64) -> String {
    let local = st.local;
    let since_midnight = local.hour as i64 * 3600 + local.minute as i64 * 60 + local.second as i64;
    let ago = (st.unix - at).max(0);
    if ago <= since_midnight {
        let secs = since_midnight - ago;
        let t = LocalDateTime { hour: (secs / 3600) as u8, minute: ((secs / 60) % 60) as u8, second: 0, ..local };
        return format!("Updated {}", fmt_clock(&t, st.settings.use_24h));
    }
    let days = (ago - since_midnight - 1) / 86_400 + 1;
    if days == 1 {
        return "Updated yesterday".into();
    }
    format!("Updated {}", fmt_date_short(&local.add_days(-days)))
}

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let accent = st.accent;
    let tab = st.store.fitness.active_tab;

    // 1. Sub-navigation tabs at the top (clean segmented style)
    let tab_w = 110.0;
    let tab_gap = 6.0;
    let nav_y = area.y + 2.0;

    let tabs = [
        ("Fasting", 0, Icon::Timer),
        ("Berat Badan", 1, Icon::Steps),
        ("Nutrisi", 2, Icon::Sliders),
    ];

    let mut tx = area.x;
    for (label, idx, icon) in tabs {
        let style = if tab == idx {
            PillStyle::Primary(accent)
        } else {
            PillStyle::Utility
        };
        let b = Rect::new(tx, nav_y, tab_w, PILL_SMALL);
        if ui.pill(wid_n("fitness.nav", idx as u64), b, label, Some(icon), style).clicked {
            st.store.fitness.active_tab = idx;
            st.save();
        }
        tx += tab_w + tab_gap;
    }

    let content_y = nav_y + PILL_SMALL + 10.0;
    let content_rect = Rect::new(area.x, content_y, area.w, area.bottom() - content_y);

    match tab {
        0 => draw_fasting(ui, st, content_rect, accent),
        1 => draw_weight(ui, st, content_rect, accent),
        _ => draw_food_calories(ui, st, content_rect, accent),
    }
}

// -------------------------------------------------------------------------------------------------
// 1. INTERMITTENT FASTING (IF)
// -------------------------------------------------------------------------------------------------
fn draw_fasting(ui: &mut Ui, st: &mut AppState, r: Rect, accent: Color) {
    let now = st.unix;
    let fast = st.store.fitness.fasting.clone();
    let is_fasting = fast.is_fasting;
    let target_hours = fast.target_hours.max(1);

    // Protocol selector pills on top right
    let protocols = [("16:8", 16), ("18:6", 18), ("20:4", 20), ("24h", 24)];
    let proto_w = 46.0;
    let mut px = r.right() - (proto_w * 4.0 + 14.0);
    for (name, h) in protocols {
        let style = if target_hours == h {
            PillStyle::Primary(accent)
        } else {
            PillStyle::Utility
        };
        let b = Rect::new(px, r.y, proto_w, 22.0);
        if ui.pill(wid_n("fasting.proto", h as u64), b, name, None, style).clicked {
            st.store.fitness.fasting.target_hours = h;
            st.save();
        }
        px += proto_w + 4.0;
    }

    if is_fasting {
        let elapsed = fast.elapsed_secs(now);
        let hours = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        let secs = elapsed % 60;
        let progress = fast.progress(now);
        let pct = (progress * 100.0).round() as u32;

        let (stage_title, stage_desc) = fast.stage(now);

        // Big elapsed timer
        let timer_str = format!("{:02}:{:02}:{:02}", hours, mins, secs);
        let header = TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT);
        ui.c.text(&timer_str, Rect::new(r.x, r.y, 160.0, 24.0), &header);

        let sub_info = format!("Target {} jam · {}% tercapai", target_hours, pct);
        ui.c.text(&sub_info, Rect::new(r.x, r.y + 24.0, 240.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

        // Fasting stage badge
        let stage_box = Rect::new(r.x, r.y + 46.0, r.w, 46.0);
        ui.c.fill_rrect(stage_box, 8.0, palette::SURFACE);
        ui.c.stroke_rrect(stage_box, 8.0, palette::BORDER, 1.0);
        ui.c.text(stage_title, Rect::new(stage_box.x + 12.0, stage_box.y + 6.0, stage_box.w - 24.0, 16.0), &TextStyle::body().weight(Weight::Semibold).color(accent));
        ui.c.text(stage_desc, Rect::new(stage_box.x + 12.0, stage_box.y + 24.0, stage_box.w - 24.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

        // Action button: Stop Fast
        let btn_w = 200.0;
        let btn_r = Rect::new(r.x, r.y + 102.0, btn_w, 28.0);
        if ui.pill(wid("fasting.toggle"), btn_r, "Selesai Puasa", Some(Icon::Check), PillStyle::Neutral).clicked {
            st.store.fitness.fasting.stop(now);
            st.save();
        }
    } else {
        // Eating window open
        ui.c.text("Jendela Makan", Rect::new(r.x, r.y, 200.0, 22.0), &TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT));
        ui.c.text("Konsumsi nutrisi sehat seimbang sesuai target harian", Rect::new(r.x, r.y + 22.0, r.w, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

        let card = Rect::new(r.x, r.y + 46.0, r.w, 46.0);
        ui.c.fill_rrect(card, 8.0, palette::SURFACE);
        ui.c.stroke_rrect(card, 8.0, palette::BORDER, 1.0);
        ui.c.text("Jadwal Puasa Berikutnya", Rect::new(card.x + 12.0, card.y + 6.0, 200.0, 16.0), &TextStyle::body().weight(Weight::Semibold).color(palette::TEXT));
        let p_desc = format!("Protokol {} jam puasa. Mulai saat selesai makan terakhir.", target_hours);
        ui.c.text(&p_desc, Rect::new(card.x + 12.0, card.y + 24.0, card.w - 24.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

        // Action button: Start Fast
        let btn_w = 180.0;
        let btn_r = Rect::new(r.x, r.y + 102.0, btn_w, 28.0);
        if ui.pill(wid("fasting.toggle"), btn_r, "Mulai Puasa", Some(Icon::Play), PillStyle::Primary(accent)).clicked {
            st.store.fitness.fasting.start(now);
            st.save();
        }
    }
}

// -------------------------------------------------------------------------------------------------
// 2. WEIGHT TRACKER & PROGRESS
// -------------------------------------------------------------------------------------------------
fn draw_weight(ui: &mut Ui, st: &mut AppState, r: Rect, accent: Color) {
    let fitness = &st.store.fitness;
    let current_weight = fitness.weight_logs.last().map(|w| w.weight_kg).unwrap_or(75.0);
    let target_weight = fitness.target_weight.unwrap_or(68.0);
    let initial_weight = fitness.initial_weight.unwrap_or(current_weight);

    // Stats bar: Current | Target | Progress
    let card_w = (r.w - 16.0) / 3.0;
    let card_h = 46.0;

    // Card 1: Current Weight
    let c1 = Rect::new(r.x, r.y, card_w, card_h);
    ui.c.fill_rrect(c1, 8.0, palette::SURFACE);
    ui.c.stroke_rrect(c1, 8.0, palette::BORDER, 1.0);
    ui.c.text("Saat Ini", Rect::new(c1.x + 10.0, c1.y + 4.0, c1.w, 14.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    ui.c.text(&format!("{:.1} kg", current_weight), Rect::new(c1.x + 10.0, c1.y + 18.0, c1.w, 22.0), &TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT));

    // Card 2: Target Weight
    let c2 = Rect::new(c1.right() + 8.0, r.y, card_w, card_h);
    ui.c.fill_rrect(c2, 8.0, palette::SURFACE);
    ui.c.stroke_rrect(c2, 8.0, palette::BORDER, 1.0);
    ui.c.text("Target", Rect::new(c2.x + 10.0, c2.y + 4.0, c2.w, 14.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    ui.c.text(&format!("{:.1} kg", target_weight), Rect::new(c2.x + 10.0, c2.y + 18.0, c2.w, 22.0), &TextStyle::headline().weight(Weight::Semibold).color(accent));

    // Card 3: Total Diff
    let diff = current_weight - initial_weight;
    let c3 = Rect::new(c2.right() + 8.0, r.y, card_w, card_h);
    ui.c.fill_rrect(c3, 8.0, palette::SURFACE);
    ui.c.stroke_rrect(c3, 8.0, palette::BORDER, 1.0);
    ui.c.text("Perubahan", Rect::new(c3.x + 10.0, c3.y + 4.0, c3.w, 14.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    let diff_str = if diff <= 0.0 { format!("{:.1} kg", diff) } else { format!("+{:.1} kg", diff) };
    ui.c.text(&diff_str, Rect::new(c3.x + 10.0, c3.y + 18.0, c3.w, 22.0), &TextStyle::headline().weight(Weight::Semibold).color(palette::TEXT));

    // Quick log today's weight row
    let input_y = c1.bottom() + 10.0;
    ui.c.text("Catat Timbangan Hari Ini", Rect::new(r.x, input_y, 200.0, 16.0), &TextStyle::footnote().color(palette::TEXT_DIM));

    if st.ui.fitness_weight_input < 30.0 {
        st.ui.fitness_weight_input = current_weight;
    }

    let mut bx = r.x;
    let step_btns = [("-0.5", -0.5f32), ("-0.1", -0.1f32), ("+0.1", 0.1f32), ("+0.5", 0.5f32)];
    for (label, val) in step_btns {
        let btn_r = Rect::new(bx, input_y + 20.0, 48.0, 26.0);
        if ui.pill(wid_n("weight.step", (val * 10.0) as i64 as u64), btn_r, label, None, PillStyle::Utility).clicked {
            st.ui.fitness_weight_input = (st.ui.fitness_weight_input + val).clamp(30.0, 250.0);
        }
        bx += 52.0;
    }

    // Weight display pill
    let val_box = Rect::new(bx + 4.0, input_y + 20.0, 76.0, 26.0);
    ui.c.fill_rrect(val_box, 6.0, palette::SURFACE);
    ui.c.stroke_rrect(val_box, 6.0, palette::BORDER, 1.0);
    ui.c.text(&format!("{:.1} kg", st.ui.fitness_weight_input), Rect::new(val_box.x, val_box.y + 4.0, val_box.w, 18.0), &TextStyle::body().weight(Weight::Semibold).color(palette::TEXT).center());

    // Save button
    let save_r = Rect::new(val_box.right() + 8.0, input_y + 20.0, 100.0, 26.0);
    if ui.pill(wid("weight.save"), save_r, "Simpan", Some(Icon::Check), PillStyle::Primary(accent)).clicked {
        let today = st.local.day_key();
        let new_w = st.ui.fitness_weight_input;
        if st.store.fitness.initial_weight.is_none() {
            st.store.fitness.initial_weight = Some(new_w);
        }
        st.store.fitness.weight_logs.push(WeightEntry {
            timestamp: st.unix,
            date: today,
            weight_kg: new_w,
            note: None,
        });
        st.save();
    }

    // Target adjustment pills
    let tgt_y = input_y + 54.0;
    ui.c.text("Atur Target Berat", Rect::new(r.x, tgt_y, 130.0, 16.0), &TextStyle::caption().color(palette::TEXT_FAINT));
    if ui.pill(wid("weight.tgt.minus"), Rect::new(r.x + 130.0, tgt_y - 2.0, 50.0, 22.0), "-1 kg", None, PillStyle::Utility).clicked {
        st.store.fitness.target_weight = Some((target_weight - 1.0).max(30.0));
        st.save();
    }
    if ui.pill(wid("weight.tgt.plus"), Rect::new(r.x + 184.0, tgt_y - 2.0, 50.0, 22.0), "+1 kg", None, PillStyle::Utility).clicked {
        st.store.fitness.target_weight = Some((target_weight + 1.0).min(200.0));
        st.save();
    }
}

// -------------------------------------------------------------------------------------------------
// 3. FOOD INPUT & CALORIE ESTIMATOR
// -------------------------------------------------------------------------------------------------
fn draw_food_calories(ui: &mut Ui, st: &mut AppState, r: Rect, accent: Color) {
    let today = st.local.day_key();
    let target_cal = st.store.fitness.daily_calorie_target.max(1000);

    let (total_cal, total_prot, total_carb, total_fat, food_count) = {
        let foods = st.store.fitness.food_logs.get(&today);
        let cal = foods.map(|list| list.iter().map(|f| f.calories).sum()).unwrap_or(0);
        let prot = foods.map(|list| list.iter().map(|f| f.protein_g).sum()).unwrap_or(0.0);
        let carb = foods.map(|list| list.iter().map(|f| f.carbs_g).sum()).unwrap_or(0.0);
        let fat = foods.map(|list| list.iter().map(|f| f.fat_g).sum()).unwrap_or(0.0);
        let count = foods.map(|list| list.len()).unwrap_or(0);
        (cal, prot, carb, fat, count)
    };

    // 1. Calorie summary banner
    let cal_str = format!("Hari Ini: {} / {} kkal", total_cal, target_cal);
    let rem_cal = target_cal.saturating_sub(total_cal);
    let rem_str = if total_cal <= target_cal {
        format!("(Sisa {} kkal)", rem_cal)
    } else {
        format!("(Kelebihan {} kkal)", total_cal - target_cal)
    };

    ui.c.text(&cal_str, Rect::new(r.x, r.y, 180.0, 18.0), &TextStyle::body().weight(Weight::Semibold).color(palette::TEXT));
    ui.c.text(&rem_str, Rect::new(r.x + 184.0, r.y + 2.0, 160.0, 16.0), &TextStyle::footnote().color(palette::TEXT_FAINT));

    // Macros row
    let macro_str = format!("Protein: {:.0}g  ·  Karbo: {:.0}g  ·  Lemak: {:.0}g", total_prot, total_carb, total_fat);
    ui.c.text(&macro_str, Rect::new(r.x, r.y + 20.0, r.w, 16.0), &TextStyle::caption().color(palette::TEXT_DIM));

    // 2. Text input row for food estimation
    let input_y = r.y + 40.0;
    let input_r = Rect::new(r.x, input_y, r.w - 74.0, 28.0);
    let placeholder = "Catat makanan (cth: nasi padang, 2 telur rebus, ayam bakar, kopi susu)...";

    let resp = ui.text_input_row(
        wid("fitness.food_input"),
        input_r,
        &mut st.ui.fitness_food_input,
        placeholder,
        64,
        accent,
        Some(Icon::Plus),
    );

    let add_btn_r = Rect::new(input_r.right() + 6.0, input_y, 68.0, 28.0);
    let add_clicked = ui.pill(wid("fitness.food_add"), add_btn_r, "Tambah", None, PillStyle::Primary(accent)).clicked;

    if resp.submitted || add_clicked {
        let text = st.ui.fitness_food_input.trim();
        if !text.is_empty() {
            let (cal, prot, carb, fat) = estimate_food(text);
            let time_str = format!("{:02}:{:02}", (st.unix / 3600) % 24, (st.unix / 60) % 60);
            let id = st.unix as u64 ^ (food_count as u64);
            st.store.fitness.food_logs.entry(today.clone()).or_default().push(FoodEntry {
                id,
                time: time_str,
                name: text.to_string(),
                calories: cal,
                protein_g: prot,
                carbs_g: carb,
                fat_g: fat,
            });
            st.ui.fitness_food_input.clear();
            st.save();
        }
    }

    // 3. Logged food list
    let list_y = input_y + 34.0;
    let list_h = (r.bottom() - list_y).max(20.0);
    let list_r = Rect::new(r.x, list_y, r.w, list_h);

    if let Some(entries) = st.store.fitness.food_logs.get(&today).cloned() {
        if entries.is_empty() {
            ui.c.text("Belum ada log makanan hari ini", list_r, &TextStyle::footnote().color(palette::TEXT_FAINT));
        } else {
            let mut ly = list_y;
            for item in entries.iter().rev().take(3) {
                let row_r = Rect::new(r.x, ly, r.w, 22.0);
                ui.c.fill_rrect(row_r, 4.0, palette::SURFACE);
                let time_w = 46.0;
                ui.c.text(&item.time, Rect::new(row_r.x + 6.0, row_r.y + 3.0, time_w, 16.0), &TextStyle::caption().color(palette::TEXT_FAINT));
                let name_w = row_r.w - 180.0;
                ui.c.text(&item.name, Rect::new(row_r.x + time_w + 6.0, row_r.y + 3.0, name_w, 16.0), &TextStyle::footnote().color(palette::TEXT));
                let info = format!("{} kkal (P:{:.0}g K:{:.0}g L:{:.0}g)", item.calories, item.protein_g, item.carbs_g, item.fat_g);
                ui.c.text(&info, Rect::new(row_r.right() - 170.0, row_r.y + 3.0, 164.0, 16.0), &TextStyle::caption().color(palette::TEXT_DIM));
                ly += 24.0;
                if ly + 22.0 > r.bottom() {
                    break;
                }
            }
        }
    } else {
        ui.c.text("Belum ada log makanan hari ini", list_r, &TextStyle::footnote().color(palette::TEXT_FAINT));
    }
}
