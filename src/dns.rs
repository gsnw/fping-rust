use std::net::{IpAddr, ToSocketAddrs};

pub fn resolve(name: &str, only_ipv4: bool, only_ipv6: bool) -> Option<IpAddr> {

  if let Ok(ip) = name.parse::<IpAddr>() {
    if only_ipv4 && ip.is_ipv6() {
      return None;
    }
    if only_ipv6 && ip.is_ipv4() {
      return None;
    }
    return Some(ip);
  }

  let addrs: Vec<IpAddr> = match (name, 0u16).to_socket_addrs() {
    Ok(iter) => iter.map(|sa| sa.ip()).collect(),
    Err(_) => return None,
  };

  if only_ipv6 {
    addrs.iter().find(|a| a.is_ipv6()).copied()
      .or_else(|| addrs.first().copied())
  } else {
    addrs.iter().find(|a| a.is_ipv4()).copied()
      .or_else(|| addrs.first().copied())
  }
}