use ncurses::*;
use std::time::Instant;
use crate::types::HostEntry;

pub fn init() {
    initscr();
    cbreak();
    noecho();
    timeout(0); // non-blocking getch()
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
}

pub fn cleanup() {
    endwin();
}

pub fn update(hosts: &[HostEntry], start: Instant) -> bool {
    // Process user input (press q or ESC to exit)
    let ch = getch();
    if ch == 'q' as i32 || ch == 27 {
        return false;
    }

    clear();
    let elapsed = start.elapsed().as_secs_f64();
    mvprintw(0, 0, &format!("fping-rs TUI - Elapsed: {:.1}s (Press 'q' or ESC to quit)\n", elapsed));

    mvprintw(2, 0, &format!("{:<20} | {:>6} | {:>6} | {:>6} | {:>8} | {:>8} | {:>8} | {:>8}",
        "Host", "Sent", "Recv", "Loss%", "Min", "Avg", "Max", "Last"));
    mvprintw(3, 0, &"-".repeat(90));

    let mut row = 4;
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

        mvprintw(row, 0, &format!("{:<20} | {:>6} | {:>6} | {:>5}% | {:>8} | {:>8} | {:>8} | {:>8}",
            display, h.num_sent, h.num_recv, loss, min_s, avg_s, max_s, last_s));
        row += 1;
    }

    refresh();
    true
}