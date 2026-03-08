use libc::{
  c_void, recvfrom, sendto, sockaddr, socklen_t,
  AF_INET, AF_INET6, IPPROTO_ICMP, IPPROTO_ICMPV6, SOCK_DGRAM, SOCK_RAW,
};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::unix::io::RawFd;

use crate::constants::{
  ICMP6_ECHO_REPLY, ICMP6_ECHO_REQUEST, ICMP_ECHO_REPLY, ICMP_ECHO_REQUEST, ICMP_HEADER_LEN,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SocketKind {
    Raw,
    Dgram,
}

pub fn open_raw_socket(is_ipv6: bool) -> Result<(RawFd, SocketKind), String> {
  let (domain, proto) = if is_ipv6 {
    (AF_INET6, IPPROTO_ICMPV6)
  } else {
    (AF_INET, IPPROTO_ICMP)
  };

  let fd = unsafe { libc::socket(domain, SOCK_RAW, proto) };
  if fd >= 0 {
    set_nonblocking(fd);
    return Ok((fd, SocketKind::Raw));
  }

  let fd = unsafe { libc::socket(domain, SOCK_DGRAM, proto) };
  if fd >= 0 {
    set_nonblocking(fd);
    return Ok((fd, SocketKind::Dgram));
  }

  Err(format!(
    "Unable to open ICMP{}-socket. Run as root or with CAP_NET_RAW.",
    if is_ipv6 { "v6" } else { "" }
  ))
}

fn set_nonblocking(fd: RawFd) {
  unsafe {
    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
  }
}

pub fn build_icmp_packet(id: u16, seq: u16, data_size: usize, is_ipv6: bool) -> Vec<u8> {
  let total = ICMP_HEADER_LEN + data_size;
  let mut pkt = vec![0u8; total];

  pkt[0] = if is_ipv6 { ICMP6_ECHO_REQUEST } else { ICMP_ECHO_REQUEST };
  pkt[1] = 0;
  pkt[2] = 0;
  pkt[3] = 0;
  pkt[4] = (id >> 8) as u8;
  pkt[5] = (id & 0xFF) as u8;
  pkt[6] = (seq >> 8) as u8;
  pkt[7] = (seq & 0xFF) as u8;

  for (i, b) in pkt[8..].iter_mut().enumerate() {
    *b = (i & 0xFF) as u8;
  }

  if !is_ipv6 {
    let cksum = icmp_checksum(&pkt);
    pkt[2] = (cksum >> 8) as u8;
    pkt[3] = (cksum & 0xFF) as u8;
  }

  pkt
}

fn icmp_checksum(data: &[u8]) -> u16 {
  let mut sum: u32 = 0;
  let mut i = 0;
  while i + 1 < data.len() {
    sum += u16::from_be_bytes([data[i], data[i + 1]]) as u32;
    i += 2;
  }
  if i < data.len() {
    sum += (data[i] as u32) << 8;
  }
  while sum >> 16 != 0 {
    sum = (sum & 0xFFFF) + (sum >> 16);
  }
  !(sum as u16)
}

pub fn send_ping_v4(fd: RawFd, addr: &Ipv4Addr, pkt: &[u8]) -> bool {
  unsafe {
    let mut sa: libc::sockaddr_in = std::mem::zeroed();
    sa.sin_family = AF_INET as u16;
    sa.sin_addr.s_addr = u32::from_ne_bytes(addr.octets()).to_be();

    let n = sendto(
      fd,
      pkt.as_ptr() as *const c_void,
      pkt.len(),
      0,
      &sa as *const _ as *const sockaddr,
      std::mem::size_of::<libc::sockaddr_in>() as socklen_t,
    );
    n == pkt.len() as isize
  }
}

pub fn send_ping_v6(fd: RawFd, addr: &Ipv6Addr, pkt: &[u8]) -> bool {
  unsafe {
    let mut sa: libc::sockaddr_in6 = std::mem::zeroed();
    sa.sin6_family = AF_INET6 as u16;
    sa.sin6_addr.s6_addr = addr.octets();

    let n = sendto(
      fd,
      pkt.as_ptr() as *const c_void,
      pkt.len(),
      0,
      &sa as *const _ as *const sockaddr,
      std::mem::size_of::<libc::sockaddr_in6>() as socklen_t,
    );
    n == pkt.len() as isize
  }
}

pub struct ReceivedPing {
  pub id: u16,
  pub seq: u16,
  pub is_ipv6: bool,
  pub raw_len: usize,
}

pub fn recv_ping(fd: RawFd, buf: &mut [u8], is_ipv6: bool, kind: SocketKind) -> Option<ReceivedPing> {
  let n = unsafe {
    let mut src: libc::sockaddr_storage = std::mem::zeroed();
    let mut src_len = std::mem::size_of::<libc::sockaddr_storage>() as socklen_t;
    recvfrom(
      fd,
      buf.as_mut_ptr() as *mut c_void,
      buf.len(),
      libc::MSG_DONTWAIT,
      &mut src as *mut _ as *mut sockaddr,
      &mut src_len,
    )
  };

  if n < 0 {
    return None;
  }

  let raw_len = n as usize;
  let data = &buf[..raw_len];

  let icmp = if is_ipv6 || kind == SocketKind::Dgram {
    if data.len() < 8 { return None; }
    data
  } else {
    if data.len() < 20 + 8 { return None; }
    let ihl = ((data[0] & 0x0F) as usize) * 4;
    if data.len() < ihl + 8 { return None; }
    &data[ihl..]
  };

  let icmp_type = icmp[0];
  let is_reply = icmp_type == ICMP_ECHO_REPLY || icmp_type == ICMP6_ECHO_REPLY;
  if !is_reply {
    return None;
  }

  Some(ReceivedPing {
    id:  u16::from_be_bytes([icmp[4], icmp[5]]),
    seq: u16::from_be_bytes([icmp[6], icmp[7]]),
    is_ipv6,
    raw_len,
  })
}