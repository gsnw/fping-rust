/*
 * Integration tests for the socket module (socket.rs)
 * 
 * Run with: cargo test --test socket_tests
 */

use fping::socket::{build_icmp_packet, SocketKind};
use fping::constants::{
  ICMP6_ECHO_REQUEST, ICMP_ECHO_REQUEST, ICMP_HEADER_LEN,
};

#[test]
fn build_packet_correct_length_ipv4() {
  let pkt = build_icmp_packet(1234, 42, 56, false, SocketKind::Raw);
  assert_eq!(pkt.len(), ICMP_HEADER_LEN + 56);
}

#[test]
fn build_packet_correct_length_ipv6() {
  let pkt = build_icmp_packet(1234, 42, 56, true, SocketKind::Raw);
  assert_eq!(pkt.len(), ICMP_HEADER_LEN + 56);
}

#[test]
fn build_packet_zero_data_size() {
  let pkt = build_icmp_packet(0, 0, 0, false, SocketKind::Raw);
  assert_eq!(pkt.len(), ICMP_HEADER_LEN);
}

#[test]
fn build_packet_ipv4_type_field() {
  let pkt = build_icmp_packet(0, 0, 0, false, SocketKind::Raw);
  assert_eq!(pkt[0], ICMP_ECHO_REQUEST);
}

#[test]
fn build_packet_ipv6_type_field() {
  let pkt = build_icmp_packet(0, 0, 0, true, SocketKind::Raw);
  assert_eq!(pkt[0], ICMP6_ECHO_REQUEST);
}

#[test]
fn build_packet_id_encoded_correctly() {
  let id: u16 = 0xABCD;
  let pkt = build_icmp_packet(id, 0, 0, false, SocketKind::Raw);
  let id_read = u16::from_be_bytes([pkt[4], pkt[5]]);
  assert_eq!(id_read, id);
}

#[test]
fn build_packet_seq_encoded_correctly() {
  let seq: u16 = 0x1234;
  let pkt = build_icmp_packet(0, seq, 0, false, SocketKind::Raw);
  let seq_read = u16::from_be_bytes([pkt[6], pkt[7]]);
  assert_eq!(seq_read, seq);
}

#[test]
fn build_packet_seq_wraps_at_u16_max() {
  let pkt = build_icmp_packet(0, u16::MAX, 0, false, SocketKind::Raw);
  let seq_read = u16::from_be_bytes([pkt[6], pkt[7]]);
  assert_eq!(seq_read, u16::MAX);
}

#[test]
fn build_packet_ipv4_checksum_is_valid() {
  let pkt = build_icmp_packet(0x1234, 0x0001, 56, false, SocketKind::Raw);

  let mut sum: u32 = 0;
  let mut i = 0;
  while i + 1 < pkt.len() {
    sum += u16::from_be_bytes([pkt[i], pkt[i + 1]]) as u32;
    i += 2;
  }
  if i < pkt.len() {
    sum += (pkt[i] as u32) << 8;
  }
  while sum >> 16 != 0 {
    sum = (sum & 0xFFFF) + (sum >> 16);
  }
  assert_eq!(sum as u16, 0xFFFF, "Invalid ICMP checksum");
}

#[test]
fn build_packet_ipv6_checksum_field_is_zero() {
  let pkt = build_icmp_packet(0x1234, 0x0001, 56, true, SocketKind::Raw);
  assert_eq!(pkt[2], 0);
  assert_eq!(pkt[3], 0);
}

#[test]
fn build_packet_payload_pattern() {
  let pkt = build_icmp_packet(0, 0, 8, false, SocketKind::Raw);
  for (i, &b) in pkt[ICMP_HEADER_LEN..].iter().enumerate() {
    assert_eq!(b, (i & 0xFF) as u8, "Payload byte {} is incorrect", i);
  }
}