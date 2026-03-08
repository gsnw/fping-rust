use std::net::IpAddr;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct HostEntry {
  pub name: String,
  pub display: String,
  pub addr: IpAddr,
  pub is_ipv6: bool,

  pub num_sent: u32,
  pub num_recv: u32,
  pub max_reply: Option<Duration>,
  pub min_reply: Option<Duration>,
  pub total_time: Duration,

  pub resp_times: Vec<Option<Duration>>,

  pub last_send: Option<Instant>,
  pub next_send: Instant,
  pub retries_left: u32,
  pub current_ping_index: u32,
  pub done: bool,
}

impl HostEntry {
  pub fn new(name: String, addr: IpAddr, is_ipv6: bool, count: u32) -> Self {
    let display = name.clone();
    HostEntry {
      name,
      display,
      addr,
      is_ipv6,
      num_sent: 0,
      num_recv: 0,
      max_reply: None,
      min_reply: None,
      total_time: Duration::ZERO,
      resp_times: if count > 0 {
          vec![None; count as usize]
      } else {
          Vec::new()
      },
      last_send: None,
      next_send: Instant::now(),
      retries_left: 0,
      current_ping_index: 0,
      done: false,
    }
  }

  pub fn record_reply(&mut self, rtt: Duration, ping_index: u32) {
    self.num_recv += 1;
    self.total_time += rtt;
    self.max_reply = Some(self.max_reply.map_or(rtt, |m| m.max(rtt)));
    self.min_reply = Some(self.min_reply.map_or(rtt, |m| m.min(rtt)));
    if let Some(slot) = self.resp_times.get_mut(ping_index as usize) {
      *slot = Some(rtt);
    }
  }

  pub fn avg_reply(&self) -> Option<Duration> {
    (self.num_recv > 0).then(|| self.total_time / self.num_recv)
  }

  pub fn loss_pct(&self) -> u32 {
    if self.num_sent == 0 {
      0
    } else {
      ((self.num_sent - self.num_recv) * 100) / self.num_sent
    }
  }
}

pub struct PendingPing {
  pub host_index: usize,
  pub ping_index: u32,
  pub sent_at: Instant,
}

#[derive(Default)]
pub struct GlobalStats {
  pub num_alive: usize,
  pub num_unreachable: usize,
  pub num_pingsent: u32,
  pub num_pingreceived: u32,
  pub num_othericmprcvd: u32,
  pub num_timeout: u32,
}