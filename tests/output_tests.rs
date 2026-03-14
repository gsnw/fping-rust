/*
 * Integration tests for the output module (output.rs)
 * 
 * Run with: cargo test --test output_tests
 */

use fping::output::{max_host_len, sprint_tm};
use fping::types::HostEntry;
use std::net::IpAddr;
use std::time::Duration;

fn host(name: &str) -> HostEntry {
  let addr: IpAddr = "1.2.3.4".parse().unwrap();
  HostEntry::new(name.to_string(), addr, false, 0)
}

#[test]
fn sprint_tm_sub_millisecond() {
  let s = sprint_tm(Duration::from_micros(500));
  assert!(s.contains('.'), "Expects a decimal point: {}", s);
  let val: f64 = s.parse().expect("No float");
  assert!((val - 0.500).abs() < 0.001);
}

#[test]
fn sprint_tm_single_digit_ms() {
  let s = sprint_tm(Duration::from_millis(5));
  let val: f64 = s.parse().expect("No float");
  assert!((val - 5.0).abs() < 0.01);
}

#[test]
fn sprint_tm_double_digit_ms() {
  let s = sprint_tm(Duration::from_millis(50));
  let val: f64 = s.parse().expect("No float");
  assert!((val - 50.0).abs() < 0.1);
}

#[test]
fn sprint_tm_triple_digit_ms() {
  let s = sprint_tm(Duration::from_millis(500));
  let val: f64 = s.parse().expect("No float");
  assert!((val - 500.0).abs() < 1.0);
}

#[test]
fn sprint_tm_zero() {
  let s = sprint_tm(Duration::ZERO);
  let val: f64 = s.parse().expect("No float");
  assert_eq!(val, 0.0);
}

#[test]
fn max_host_len_empty_slice() {
  assert_eq!(max_host_len(&[]), 0);
}

#[test]
fn max_host_len_single() {
  let hosts = vec![host("example.com")];
  assert_eq!(max_host_len(&hosts), "example.com".len());
}

#[test]
fn max_host_len_multiple() {
  let hosts = vec![
    host("a.de"),
    host("very-long-hostname.example.org"),
    host("b.com"),
  ];
  assert_eq!(max_host_len(&hosts), "very-long-hostname.example.org".len());
}

#[test]
fn max_host_len_uses_display_field() {
  let addr: IpAddr = "1.2.3.4".parse().unwrap();
  let mut h = HostEntry::new("example.com".to_string(), addr, false, 0);
  h.display = "1.2.3.4".to_string();
  assert_eq!(max_host_len(&[h]), "1.2.3.4".len());
}