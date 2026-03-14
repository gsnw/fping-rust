/*
 * Integration tests for the Args module (args.rs)
 * 
 * Run with: cargo test --test args_tests
 */

use clap::Parser;
use fping::args::Args;

fn parse(argv: &[&str]) -> Args {
  Args::parse_from(argv)
}

#[test]
fn defaults_are_correct() {
  let a = parse(&["fping", "1.2.3.4"]);
  assert_eq!(a.interval, 10);
  assert_eq!(a.period, 1000);
  assert_eq!(a.timeout, 500);
  assert_eq!(a.retry, 3);
  assert!((a.backoff - 1.5).abs() < f64::EPSILON);
  assert_eq!(a.size, 56);
  assert!(!a.r#loop);
  assert!(!a.quiet);
  assert!(!a.alive);
  assert!(!a.unreach);
  assert!(!a.stats);
  assert!(!a.json);
  assert!(!a.ipv4);
  assert!(!a.ipv6);
  assert!(!a.addr);
  assert!(!a.timestamp);
}

#[test]
fn effective_count_none_by_default() {
  let a = parse(&["fping", "1.2.3.4"]);
  assert!(a.effective_count().is_none());
}

#[test]
fn effective_count_uses_count() {
  let a = parse(&["fping", "-c", "5", "1.2.3.4"]);
  assert_eq!(a.effective_count(), Some(5));
}

#[test]
fn effective_count_uses_vcount() {
  let a = parse(&["fping", "-C", "3", "1.2.3.4"]);
  assert_eq!(a.effective_count(), Some(3));
}

#[test]
fn effective_count_vcount_overrides_count() {
  let a = parse(&["fping", "-C", "7", "1.2.3.4"]);
  assert_eq!(a.effective_count(), Some(7));
  assert!(a.is_verbose_count());
}

#[test]
fn is_verbose_count_false_without_vcount() {
  let a = parse(&["fping", "-c", "4", "1.2.3.4"]);
  assert!(!a.is_verbose_count());
}

#[test]
fn is_verbose_count_true_with_vcount() {
  let a = parse(&["fping", "-C", "4", "1.2.3.4"]);
  assert!(a.is_verbose_count());
}

#[test]
fn flag_alive() {
  let a = parse(&["fping", "-a", "1.2.3.4"]);
  assert!(a.alive);
}

#[test]
fn flag_unreach() {
  let a = parse(&["fping", "-u", "1.2.3.4"]);
  assert!(a.unreach);
}

#[test]
fn flag_quiet() {
  let a = parse(&["fping", "-q", "1.2.3.4"]);
  assert!(a.quiet);
}

#[test]
fn flag_stats() {
  let a = parse(&["fping", "-s", "1.2.3.4"]);
  assert!(a.stats);
}

#[test]
fn flag_loop() {
  let a = parse(&["fping", "-l", "1.2.3.4"]);
  assert!(a.r#loop);
}

#[test]
fn flag_ipv4() {
  let a = parse(&["fping", "-4", "1.2.3.4"]);
  assert!(a.ipv4);
}

#[test]
fn flag_ipv6() {
  let a = parse(&["fping", "-6", "::1"]);
  assert!(a.ipv6);
}

#[test]
fn flag_json() {
  let a = parse(&["fping", "-J", "-c", "1", "1.2.3.4"]);
  assert!(a.json);
}

#[test]
fn flag_addr() {
  let a = parse(&["fping", "-A", "1.2.3.4"]);
  assert!(a.addr);
}

#[test]
fn flag_timestamp() {
  let a = parse(&["fping", "-D", "1.2.3.4"]);
  assert!(a.timestamp);
}

#[test]
fn option_interval() {
  let a = parse(&["fping", "-i", "25", "1.2.3.4"]);
  assert_eq!(a.interval, 25);
}

#[test]
fn option_period() {
  let a = parse(&["fping", "-p", "2000", "1.2.3.4"]);
  assert_eq!(a.period, 2000);
}

#[test]
fn option_timeout() {
  let a = parse(&["fping", "-t", "1000", "1.2.3.4"]);
  assert_eq!(a.timeout, 1000);
}

#[test]
fn option_retry() {
  let a = parse(&["fping", "-r", "5", "1.2.3.4"]);
  assert_eq!(a.retry, 5);
}

#[test]
fn option_size() {
  let a = parse(&["fping", "-b", "128", "1.2.3.4"]);
  assert_eq!(a.size, 128);
}

#[test]
fn option_reachable() {
  let a = parse(&["fping", "-x", "2", "1.2.3.4", "2.3.4.5", "3.4.5.6"]);
  assert_eq!(a.reachable, Some(2));
}

#[test]
fn multiple_targets() {
  let a = parse(&["fping", "1.1.1.1", "8.8.8.8", "9.9.9.9"]);
  assert_eq!(a.targets.len(), 3);
  assert_eq!(a.targets[0], "1.1.1.1");
  assert_eq!(a.targets[1], "8.8.8.8");
  assert_eq!(a.targets[2], "9.9.9.9");
}

#[test]
fn option_file() {
  let a = parse(&["fping", "-f", "/tmp/hosts.txt"]);
  assert_eq!(a.file.as_deref(), Some("/tmp/hosts.txt"));
}

#[test]
fn option_file_stdin() {
  let a = parse(&["fping", "-f", "-"]);
  assert_eq!(a.file.as_deref(), Some("-"));
}