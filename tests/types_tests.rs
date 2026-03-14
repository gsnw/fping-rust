/*
 * Integration tests for the Types module (types.rs)
 * 
 * Run with: cargo test --test types_tests
 */

use fping::types::HostEntry;
use std::net::IpAddr;
use std::time::Duration;

fn make_host(count: u32) -> HostEntry {
  let addr: IpAddr = "1.2.3.4".parse().unwrap();
  HostEntry::new("example.com".to_string(), addr, false, count)
}

#[test]
fn new_host_defaults() {
  let h = make_host(0);
  assert_eq!(h.num_sent, 0);
  assert_eq!(h.num_recv, 0);
  assert!(h.min_reply.is_none());
  assert!(h.max_reply.is_none());
  assert_eq!(h.total_time, Duration::ZERO);
  assert!(!h.done);
  assert_eq!(h.current_ping_index, 0);
}

#[test]
fn new_host_with_count_preallocates_resp_times() {
  let h = make_host(5);
  assert_eq!(h.resp_times.len(), 5);
  assert!(h.resp_times.iter().all(|r| r.is_none()));
}

#[test]
fn record_reply_increments_recv() {
  let mut h = make_host(3);
  h.record_reply(Duration::from_millis(10), 0);
  assert_eq!(h.num_recv, 1);
}

#[test]
fn record_reply_updates_min_max() {
  let mut h = make_host(3);
  h.record_reply(Duration::from_millis(20), 0);
  h.record_reply(Duration::from_millis(5), 1);
  h.record_reply(Duration::from_millis(50), 2);

  assert_eq!(h.min_reply, Some(Duration::from_millis(5)));
  assert_eq!(h.max_reply, Some(Duration::from_millis(50)));
}

#[test]
fn record_reply_stores_in_resp_times() {
  let mut h = make_host(3);
  h.record_reply(Duration::from_millis(42), 1);
  assert_eq!(h.resp_times[1], Some(Duration::from_millis(42)));
  assert!(h.resp_times[0].is_none());
  assert!(h.resp_times[2].is_none());
}

#[test]
fn record_reply_out_of_bounds_index_ignored() {
  let mut h = make_host(2);
  h.record_reply(Duration::from_millis(10), 99);
  assert_eq!(h.num_recv, 1);
}

#[test]
fn avg_reply_none_when_no_replies() {
  let h = make_host(0);
  assert!(h.avg_reply().is_none());
}

#[test]
fn avg_reply_correct() {
  let mut h = make_host(3);
  h.record_reply(Duration::from_millis(10), 0);
  h.record_reply(Duration::from_millis(20), 1);
  h.record_reply(Duration::from_millis(30), 2);
  assert_eq!(h.avg_reply(), Some(Duration::from_millis(20)));
}

#[test]
fn loss_pct_zero_when_no_sends() {
  let h = make_host(0);
  assert_eq!(h.loss_pct(), 0);
}

#[test]
fn loss_pct_100_when_nothing_received() {
  let mut h = make_host(0);
  h.num_sent = 4;
  assert_eq!(h.loss_pct(), 100);
}

#[test]
fn loss_pct_50() {
  let mut h = make_host(0);
  h.num_sent = 4;
  h.record_reply(Duration::from_millis(10), 0);
  h.record_reply(Duration::from_millis(10), 1);
  assert_eq!(h.loss_pct(), 50);
}

#[test]
fn loss_pct_0_all_received() {
  let mut h = make_host(3);
  h.num_sent = 3;
  h.record_reply(Duration::from_millis(10), 0);
  h.record_reply(Duration::from_millis(10), 1);
  h.record_reply(Duration::from_millis(10), 2);
  assert_eq!(h.loss_pct(), 0);
}