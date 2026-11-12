//! The AI tab while an agent waits on an answer: the question at page size,
//! one button per answer, and who is asking.
//!
//! The island's notice is only the door (`InlineAction::Answer` opens this
//! page); the answer itself is given here, on buttons big enough to read, and
//! goes back to the agent's script through `Command::AnswerAsk`.

use crate::commands::Command;
use crate::gfx::{TextStyle, Weight};
use crate::icons::Icon;
use crate::state::AppState;
use crate::ui::{wid_n, PillStyle, Ui};
use poni_core::geom::{palette, Point, Rect};

pub const HEIGHT: f32 = 236.0;
/// How many lines the question may take before the rest is cut.
const LINES: usize = 3;
const LINE_H: f32 = 25.0;
const BTN_H: f32 = 34.0;
const BTN_MIN_W: f32 = 132.0;
const BTN_GAP: f32 = 12.0;

pub fn height(_st: &AppState) -> f32 {
    HEIGHT
}

/// Words, broken to fit `max_w`, up to `LINES` lines.
fn wrap(ui: &mut Ui, text: &str, style: &TextStyle, max_w: f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let fits = if cur.is_empty() {
            ui.c.measure(word, style, 4000.0).0 <= max_w
        } else {
            ui.c.measure(&format!("{cur} {word}"), style, 4000.0).0 <= max_w
        };
        if fits {
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        } else if cur.is_empty() {
            cur = word.to_string(); // a word longer than the line: let it clip
        } else {
            lines.push(std::mem::take(&mut cur));
            cur.push_str(word);
        }
        if lines.len() == LINES {
            break;
        }
    }
    if !cur.is_empty() && lines.len() < LINES {
        lines.push(cur);
    }
    lines
}

pub fn draw(ui: &mut Ui, st: &mut AppState, area: Rect) {
    let area = Rect::new(area.x, area.y, area.w, HEIGHT);
    let Some(q) = crate::state::ask() else {
        return;
    };
    let accent = st.accent;

    // Who is asking, then the question, at the size a page allows.
    let label = TextStyle::caption().weight(Weight::Bold).color(accent);
    ui.c.text("AI ASKS YOU", Rect::new(area.x, area.y + 18.0, area.w, 16.0), &label);
    let who = TextStyle::footnote().color(palette::TEXT_DIM);
    ui.c.text(&format!("{} is waiting for your answer", q.title), Rect::new(area.x, area.y + 36.0, area.w, 16.0), &who);

    let style = TextStyle::headline().weight(Weight::Bold);
    let lines = wrap(ui, &q.question, &style, area.w);
    for (i, line) in lines.iter().enumerate() {
        ui.c.text(line, Rect::new(area.x, area.y + 64.0 + i as f32 * LINE_H, area.w, LINE_H), &style);
    }

    // One button per answer, left to right under the question.
    let mut x = area.x;
    let y = area.y + 64.0 + LINES as f32 * LINE_H + 14.0;
    for (i, option) in q.options.iter().enumerate() {
        let w = ui.pill_width(option, false, false).max(BTN_MIN_W) + 16.0;
        let r = Rect::new(x, y, w, BTN_H);
        let pill = Rect::new(r.x, r.center().y - crate::ui::PILL_SMALL / 2.0, r.w, crate::ui::PILL_SMALL);
        let look = if i == 0 { PillStyle::Primary(accent) } else { PillStyle::Utility };
        if ui.pill(wid_n("ask.opt", i as u64), pill, option, None, look).clicked {
            st.cmds.push(Command::AnswerAsk(i));
        }
        x += w + BTN_GAP;
        if x > area.right() {
            break;
        }
    }

    // The agent is blocked until this is answered: say so, quietly.
    let hint = TextStyle::footnote().color(palette::TEXT_DIM);
    let hint_y = y + BTN_H + 16.0;
    ui.c.text("The agent's script is waiting: the answer it gets is the button you press.", Rect::new(area.x, hint_y, area.w, 16.0), &hint);

    // A small mark of what this is, on the right of the header.
    let c = Point::new(area.right() - 18.0, area.y + 24.0);
    ui.c.fill_circle(c, 16.0, accent.with_alpha(0.18));
    ui.c.icon(Icon::Ai, Rect::from_center(c, 18.0, 18.0), accent);
}
