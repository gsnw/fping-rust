use libc::{
  c_void, recvfrom, sendmsg, sendto, sockaddr, socklen_t,
  AF_INET, AF_INET6, IPPROTO_ICMP, IPPROTO_ICMPV6, SOCK_DGRAM, SOCK_RAW,
};
use std::io::ErrorKind;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::os::unix::io::RawFd;
use std::ffi::CString;

use crate::constants::{
  ICMP6_ECHO_REPLY, ICMP6_ECHO_REQUEST, ICMP_ECHO_REPLY, ICMP_ECHO_REQUEST, ICMP_HEADER_LEN,
  ICMP_TYPE_OFFSET, ICMP_ID_OFFSET, ICMP_SEQ_OFFSET, ICMP_MIN_LEN, IPV4_MIN_HDR_LEN,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SocketKind {
  Raw,
  Dgram,
}

pub fn open_raw_socket(is_ipv6: bool) -> Result<(RawFd, SocketKind, Option<u16>), String> {
  let (domain, proto) = if is_ipv6 {
    (AF_INET6, IPPROTO_ICMPV6)
  } else {
    (AF_INET, IPPROTO_ICMP)
  };

  let fd = unsafe { libc::socket(domain, SOCK_RAW, proto) };
  if fd >= 0 {
    if let Err(e) = set_nonblocking(fd) {
      eprintln!("Warning: set_nonblocking failed for RAW socket: {}", e);
    }
    return Ok((fd, SocketKind::Raw, None));
  }

  let fd = unsafe { libc::socket(domain, SOCK_DGRAM, proto) };
  if fd >= 0 {
    if let Err(e) = set_nonblocking(fd) {
      eprintln!("Warning: set_nonblocking failed for DGRAM socket: {}", e);
    }

    match dgram_bind_and_get_id(fd, is_ipv6) {
      Ok(assigned_id) => return Ok((fd, SocketKind::Dgram, assigned_id)),
      Err(e) => {
        unsafe { libc::close(fd) };
        return Err(format!("DGRAM socket bind/getsockname failed: {}", e));
      }
    }
  }

  Err(format!(
    "Unable to open ICMP{} socket (no root and SOCK_DGRAM denied).\n\
    Fix with one of:\n\
    \x20 sudo setcap cap_net_raw+ep ./fping\n\
    \x20 sudo sysctl -w net.ipv4.ping_group_range=\"0 2147483647\"",
    if is_ipv6 { "v6" } else { "" }
  ))
}

fn dgram_bind_and_get_id(fd: RawFd, is_ipv6: bool) -> Result<Option<u16>, std::io::Error> {
  unsafe {
    if is_ipv6 {
      let mut sa: libc::sockaddr_in6 = std::mem::zeroed();
      sa.sin6_family = AF_INET6 as libc::sa_family_t;
      let r = libc::bind(
        fd,
        &sa as *const _ as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in6>() as socklen_t,
      );
      if r < 0 {
        return Err(std::io::Error::last_os_error());
      }

      let mut sa2: libc::sockaddr_in6 = std::mem::zeroed();
      let mut len = std::mem::size_of::<libc::sockaddr_in6>() as socklen_t;
      let r = libc::getsockname(fd, &mut sa2 as *mut _ as *mut libc::sockaddr, &mut len);
      if r < 0 {
        return Err(std::io::Error::last_os_error());
      }
      if sa2.sin6_port == 0 {
        return Ok(None);
      }
      Ok(Some(u16::from_be(sa2.sin6_port)))
    } else {
      let mut sa: libc::sockaddr_in = std::mem::zeroed();
      sa.sin_family = AF_INET as libc::sa_family_t;
      let r = libc::bind(
        fd,
        &sa as *const _ as *const libc::sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as socklen_t,
      );
      if r < 0 {
        return Err(std::io::Error::last_os_error());
      }

      let mut sa2: libc::sockaddr_in = std::mem::zeroed();
      let mut len = std::mem::size_of::<libc::sockaddr_in>() as socklen_t;
      let r = libc::getsockname(fd, &mut sa2 as *mut _ as *mut libc::sockaddr, &mut len);
      if r < 0 {
        return Err(std::io::Error::last_os_error());
      }
      if sa2.sin_port == 0 {
        return Ok(None);
      }
      Ok(Some(u16::from_be(sa2.sin_port)))
    }
  }
}

fn set_nonblocking(fd: RawFd) -> Result<(), std::io::Error> {
  unsafe {
    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
    if flags < 0 {
      return Err(std::io::Error::last_os_error());
    }
    let r = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    if r < 0 {
      return Err(std::io::Error::last_os_error());
    }
    Ok(())
  }
}

pub fn set_outgoing_iface_v4(fd: RawFd, iface: &str) -> Result<u32, String> {
  let cname = CString::new(iface).map_err(|_| format!("Invalid interface name: {}", iface))?;
  let idx = unsafe { libc::if_nametoindex(cname.as_ptr()) };
  if idx == 0 {
    return Err(format!("fping: unknown interface '{}'", iface));
  }

  let on: libc::c_int = 1;
  let r = unsafe {
    libc::setsockopt(
      fd,
      libc::IPPROTO_IP,
      libc::IP_PKTINFO,
      &on as *const _ as *const libc::c_void,
      std::mem::size_of::<libc::c_int>() as socklen_t,
    )
  };

  if r < 0 {
    return Err(format!("setsockopt IP_PKTINFO failed: {}", std::io::Error::last_os_error()));
  }

  Ok(idx)
}

pub fn set_outgoing_iface_v6(fd: RawFd, iface: &str) -> Result<u32, String> {
  let cname = CString::new(iface).map_err(|_| format!("Invalid interface name: {}", iface))?;

  let idx = unsafe { libc::if_nametoindex(cname.as_ptr()) };
  if idx == 0 {
    return Err(format!("fping: unknown interface '{}'", iface));
  }

  let _ = unsafe {
    libc::setsockopt(
      fd,
      libc::IPPROTO_IPV6,
      libc::IPV6_MULTICAST_IF,
      &idx as *const _ as *const libc::c_void,
      std::mem::size_of::<u32>() as socklen_t,
    )
  };

  Ok(idx)
}

pub fn build_icmp_packet(id: u16, seq: u16, data_size: usize, is_ipv6: bool, kind: SocketKind) -> Vec<u8> {
  let total = ICMP_HEADER_LEN + data_size;
  let mut pkt = vec![0u8; total];

  pkt[ICMP_TYPE_OFFSET] = if is_ipv6 { ICMP6_ECHO_REQUEST } else { ICMP_ECHO_REQUEST };
  pkt[1] = 0; // Code
  pkt[2] = 0; // Checksum high
  pkt[3] = 0; // Checksum low
  pkt[ICMP_ID_OFFSET] = (id >> 8) as u8;
  pkt[ICMP_ID_OFFSET + 1] = (id & 0xFF) as u8;
  pkt[ICMP_SEQ_OFFSET] = (seq >> 8) as u8;
  pkt[ICMP_SEQ_OFFSET + 1] = (seq & 0xFF) as u8;

  for (i, b) in pkt[ICMP_MIN_LEN..].iter_mut().enumerate() {
    *b = (i & 0xFF) as u8;
  }

  if !is_ipv6 {
    let cksum = icmp_checksum(&pkt);
    pkt[2] = (cksum >> 8) as u8;
    pkt[3] = (cksum & 0xFF) as u8;
  } else if kind == SocketKind::Raw {
    eprintln!("Warning: ICMPv6 checksum not computed for RAW socket – packet may be dropped");
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

pub fn send_ping_v4(fd: RawFd, addr: &Ipv4Addr, pkt: &[u8], iface_idx: Option<u32>) -> bool {
  unsafe {
    let mut sa: libc::sockaddr_in = std::mem::zeroed();
    sa.sin_family = AF_INET as libc::sa_family_t;
    sa.sin_addr.s_addr = u32::from_ne_bytes(addr.octets());

    let n = if let Some(idx) = iface_idx {
      let iov = libc::iovec {
      iov_base: pkt.as_ptr() as *mut c_void,
      iov_len: pkt.len(),
      };
      let cmsg_space = libc::CMSG_SPACE(std::mem::size_of::<libc::in_pktinfo>() as u32) as usize;
      let mut cmsg_buf = vec![0u8; cmsg_space];

      let mut msg: libc::msghdr = std::mem::zeroed();
      msg.msg_name = &sa as *const _ as *mut libc::c_void;
      msg.msg_namelen = std::mem::size_of::<libc::sockaddr_in>() as socklen_t;
      msg.msg_iov = &iov as *const _ as *mut libc::iovec;
      msg.msg_iovlen = 1;
      msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
      msg.msg_controllen = cmsg_space as _;

      let cmsg = libc::CMSG_FIRSTHDR(&msg);
      (*cmsg).cmsg_level = libc::IPPROTO_IP;
      (*cmsg).cmsg_type = libc::IP_PKTINFO;
      (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::in_pktinfo>() as u32) as _;

      let pktinfo = libc::CMSG_DATA(cmsg) as *mut libc::in_pktinfo;
      std::ptr::write_bytes(pktinfo, 0, 1);
      (*pktinfo).ipi_ifindex = idx as _;

      sendmsg(fd, &msg, 0)
    } else {
      sendto(
        fd,
        pkt.as_ptr() as *const c_void,
        pkt.len(),
        0,
        &sa as *const _ as *const sockaddr,
        std::mem::size_of::<libc::sockaddr_in>() as socklen_t,
      )
    };
    n == pkt.len() as isize
  }
}

pub fn send_ping_v6(fd: RawFd, addr: &Ipv6Addr, pkt: &[u8], iface_idx: Option<u32>) -> bool {
  unsafe {
    let mut sa: libc::sockaddr_in6 = std::mem::zeroed();
    sa.sin6_family = AF_INET6 as libc::sa_family_t;
    sa.sin6_addr.s6_addr = addr.octets();

    let n = if let Some(idx) = iface_idx {
      let iov = libc::iovec {
        iov_base: pkt.as_ptr() as *mut c_void,
        iov_len: pkt.len(),
      };
      let cmsg_space = libc::CMSG_SPACE(std::mem::size_of::<libc::in6_pktinfo>() as u32) as usize;
      let mut cmsg_buf = vec![0u8; cmsg_space];

      let mut msg: libc::msghdr = std::mem::zeroed();
      msg.msg_name = &sa as *const _ as *mut libc::c_void;
      msg.msg_namelen = std::mem::size_of::<libc::sockaddr_in6>() as socklen_t;
      msg.msg_iov = &iov as *const _ as *mut libc::iovec;
      msg.msg_iovlen = 1;
      msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
      msg.msg_controllen = cmsg_space as _;

      let cmsg = libc::CMSG_FIRSTHDR(&msg);
      (*cmsg).cmsg_level = libc::IPPROTO_IPV6;
      (*cmsg).cmsg_type = libc::IPV6_PKTINFO;
      (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::in6_pktinfo>() as u32) as _;

      let pktinfo = libc::CMSG_DATA(cmsg) as *mut libc::in6_pktinfo;
      std::ptr::write_bytes(pktinfo, 0, 1);
      (*pktinfo).ipi6_ifindex = idx;

      sendmsg(fd, &msg, 0)
    } else {
      sendto(
        fd,
        pkt.as_ptr() as *const c_void,
        pkt.len(),
        0,
        &sa as *const _ as *const sockaddr,
        std::mem::size_of::<libc::sockaddr_in6>() as socklen_t,
      )
    };
    n == pkt.len() as isize
  }
}

pub struct ReceivedPing {
  pub seq: u16,
  pub raw_len: usize,
}

fn parse_icmp_packet(data: &[u8], is_ipv6: bool, kind: SocketKind, expected_id: Option<u16>) -> Option<u16> {
  let icmp = if !is_ipv6 && kind == SocketKind::Raw {
    if data.len() < IPV4_MIN_HDR_LEN + ICMP_MIN_LEN {
      return None;
    }
    let ihl = ((data[0] & 0x0F) as usize) * 4;
    if data.len() < ihl + ICMP_MIN_LEN {
      return None;
    }
    &data[ihl..]
  } else {
    if data.len() < ICMP_MIN_LEN {
      return None;
    }
    data
  };

  let icmp_type = icmp[ICMP_TYPE_OFFSET];

  let is_reply = if is_ipv6 {
    icmp_type == ICMP6_ECHO_REPLY
  } else {
    icmp_type == ICMP_ECHO_REPLY
  };

  if !is_reply {
    return None;
  }

  let id = u16::from_be_bytes([icmp[ICMP_ID_OFFSET], icmp[ICMP_ID_OFFSET + 1]]);
  if let Some(eid) = expected_id {
    if id != eid {
      return None;
    }
  }

  Some(u16::from_be_bytes([icmp[ICMP_SEQ_OFFSET], icmp[ICMP_SEQ_OFFSET + 1]]))
}

pub fn recv_ping(fd: RawFd, buf: &mut [u8], is_ipv6: bool, kind: SocketKind, expected_id: Option<u16>) -> Option<ReceivedPing> {
  let mut src: libc::sockaddr_storage = unsafe { std::mem::zeroed() };
  let mut src_len = std::mem::size_of::<libc::sockaddr_storage>() as socklen_t;

  let n = unsafe {
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
    let err = std::io::Error::last_os_error();
    if err.kind() != ErrorKind::WouldBlock {
      eprintln!("recv_ping: recvfrom error: {}", err);
    }
    return None;
  }

  let raw_len = n as usize;
  let data = &buf[..raw_len];
  let seq = parse_icmp_packet(data, is_ipv6, kind, expected_id)?;

  Some(ReceivedPing { seq, raw_len })
}