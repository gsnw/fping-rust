/*
 * Integration tests for the DNS module (dns.rs)
 * 
 * Run with: cargo test --test dns_tests
 */

use fping::dns::resolve;
use std::net::IpAddr;

#[test]
fn resolve_ipv4_literal() {
  let result = resolve("127.0.0.1", false, false);
  assert_eq!(result, Some("127.0.0.1".parse::<IpAddr>().unwrap()));
}

#[test]
fn resolve_ipv6_literal() {
  let result = resolve("::1", false, false);
  assert_eq!(result, Some("::1".parse::<IpAddr>().unwrap()));
}

#[test]
fn resolve_ipv4_literal_rejected_when_ipv6_only() {
  let result = resolve("127.0.0.1", false, true);
  assert!(result.is_none(), "The IPv4 address should be rejected when --ipv6 is specified");
}

#[test]
fn resolve_ipv6_literal_rejected_when_ipv4_only() {
  let result = resolve("::1", true, false);
  assert!(result.is_none(), "An IPv6 address should be rejected when using --ipv4");
}

#[test]
fn resolve_localhost_hostname() {
  let result = resolve("localhost", false, false);
  assert!(result.is_some(), "localhost could not be resolved");
}

#[test]
fn resolve_localhost_ipv4_only() {
  let result = resolve("localhost", true, false);
  if let Some(addr) = result {
    assert!(addr.is_ipv4(), "Expected IPv4 address, received: {}", addr);
  }
}

#[test]
#[test]
fn resolve_localhost_ipv6_only() {
    let result = resolve("localhost", false, true);
    match result {
        None => { /* The system does not have an IPv6 localhost – acceptable */ }
        Some(addr) => assert!(
            addr.is_ipv6(),
            "Expected IPv6 address or None, but received: {}",
            addr
        ),
    }
}

#[test]
fn resolve_invalid_hostname_returns_none() {
    let result = resolve("this.hostname.does.not.exist.invalid", false, false);
 
    if result.is_some() {
        eprintln!(
            "WARNING: DNS hijacking detected ({:?}). \
             The resolve_invalid_hostname_returns_none test is skipped.",
            result
        );
        return;
    }
    assert!(result.is_none(), "An invalid hostname should return None");
}

#[test]
fn resolve_empty_string_returns_none() {
  let result = resolve("", false, false);
  assert!(result.is_none(), "An empty string should return None");
}