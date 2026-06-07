use cursive::views::TextView;
use cursive::view::Nameable;
use cursive::event::Key;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::types::HostEntry;
use chrono::Local;

#[derive(Clone, Copy)]
pub enum TuiAction {
  Continue,
  Quit,
  Reset,
}

pub struct TuiState {
  pub runner: cursive::CursiveRunner<cursive::CursiveRunnable>,
  action: Arc<Mutex<TuiAction>>,
}

pub fn init() -> TuiState {
  let mut siv = cursive::default();
  let action = Arc::new(Mutex::new(TuiAction::Continue));

  let act_q = action.clone();
  siv.add_global_callback('q', move |_| *act_q.lock().unwrap() = TuiAction::Quit);

  let act_esc = action.clone();
  siv.add_global_callback(Key::Esc, move |_| *act_esc.lock().unwrap() = TuiAction::Quit);

  let act_r = action.clone();
  siv.add_global_callback('r', move |_| *act_r.lock().unwrap() = TuiAction::Reset);

  let act_r_upper = action.clone();
  siv.add_global_callback('R', move |_| *act_r_upper.lock().unwrap() = TuiAction::Reset);

  siv.add_layer(TextView::new("Initializing...").with_name("main_view"));

  let runner = siv.into_runner();
  TuiState { runner, action }
}

pub fn cleanup(_state: TuiState) {

}

pub fn update(state: &mut TuiState, hosts: &[HostEntry], start: Instant) -> TuiAction {
  state.runner.step();

  let current_action = {
    let mut act = state.action.lock().unwrap();
    let val = *act;
    if let TuiAction::Reset = val { *act = TuiAction::Continue; }
    val
  };

  if !state.runner.is_running() { return TuiAction::Quit; }
  if let TuiAction::Quit = current_action { return current_action; }

  let elapsed = start.elapsed().as_secs_f64();
  let mut buf = String::new();

  buf.push_str("=======================================================================================================\n");
  buf.push_str(&format!(" fping-rs TUI | Hosts: {} | Elapsed: {:.1}s\n", hosts.len(), elapsed));
  buf.push_str(" Controls: [q] / [ESC] Quit TUI  |  [r] Reset Statistics\n");
  buf.push_str("=======================================================================================================\n\n");

  buf.push_str(&format!("{:<20} | {:>6} | {:>6} | {:>5}% | {:>8} | {:>8} | {:>8} | {:>8} | {:<10}\n",
    "Host", "Sent", "Recv", "Loss", "Min", "Avg", "Max", "Last", "Trend"));
  buf.push_str(&format!("{}\n", "-".repeat(103)));

  for h in hosts {
    let loss = h.loss_pct();

    let min_s = h.min_reply.map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0)).unwrap_or_else(|| "-".into());
    let avg_s = h.avg_reply().map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0)).unwrap_or_else(|| "-".into());
    let max_s = h.max_reply.map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0)).unwrap_or_else(|| "-".into());

    let last_val = h.resp_times.iter().rev().find_map(|&r| r);
    let last_s = last_val.map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0)).unwrap_or_else(|| "-".into());

    let mut display = h.display.clone();
    if display.len() > 20 {
      display.truncate(17);
      display.push_str("...");
    }

    let trend_chars = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut trend = String::new();
    let max_ms = h.max_reply.map(|d| d.as_secs_f64()).unwrap_or(0.001);
    let recent = h.resp_times.iter().skip(h.resp_times.len().saturating_sub(10));

    for r in recent {
      if let Some(d) = r {
        let ms = d.as_secs_f64();
        let mut idx = ((ms / max_ms) * (trend_chars.len() as f64 - 1.0)).round() as usize;
        if idx >= trend_chars.len() { idx = trend_chars.len() - 1; }
        trend.push(trend_chars[idx]);
      } else {
        trend.push('x');
      }
    }

    buf.push_str(&format!("{:<20} | {:>6} | {:>6} | {:>5}% | {:>8} | {:>8} | {:>8} | {:>8} | {:<10}\n",
      display, h.num_sent, h.num_recv, loss, min_s, avg_s, max_s, last_s, trend));
  }

  buf.push_str("\n=======================================================================================================\n");
  buf.push_str(&format!(" Updated: {} | Interval: 200ms | Hosts: {}/{} visible\n", Local::now().format("%H:%M:%S"), hosts.len(), hosts.len()));

  state.runner.call_on_name("main_view", |view: &mut TextView| {
    view.set_content(buf);
  });

  state.runner.refresh();
  current_action
}