use std::collections::HashMap;
use std::net::IpAddr;
use std::time::{Duration, Instant};

use crate::args::Args;
use crate::output::{
  print_alive, print_unreachable,
  print_global_stats, print_per_host_stats, print_recv, print_timeout,
  max_host_len, GlobalStatsSummary, RecvLineOpts, TimeoutLineOpts,
};
use crate::socket::{build_icmp_packet, open_raw_socket, recv_ping, send_ping_v4, send_ping_v6, SocketKind};
use crate::types::{HostEntry, PendingPing};

pub fn run(args: Args, hosts_in: Vec<(String, IpAddr)>) {
  let count = args.effective_count();
  let loop_mode = args.r#loop;
  let verbose_count = args.is_verbose_count();

  let mut hosts: Vec<HostEntry> = hosts_in
    .into_iter()
    .map(|(name, addr)| {
      let is_ipv6 = addr.is_ipv6();
      let display = if args.addr { addr.to_string() } else { name.clone() };
      let mut h = HostEntry::new(name, addr, is_ipv6, count.unwrap_or(0));
      h.display = display;
      h
    })
    .collect();

  let has_v4 = hosts.iter().any(|h| !h.is_ipv6);
  let has_v6 = hosts.iter().any(|h| h.is_ipv6);

  let (fd4, kind4, dgram_id4) = if has_v4 {
    let (fd, kind, kid) = open_raw_socket(false).unwrap_or_else(|e| {
      eprintln!("fping: {}", e);
      std::process::exit(3);
    });
    (Some(fd), kind, kid)
  } else {
    (None, SocketKind::Raw, None)
  };

  let (fd6, kind6, dgram_id6) = if has_v6 {
    let (fd, kind, kid) = open_raw_socket(true).unwrap_or_else(|e| {
      eprintln!("fping: {}", e);
      std::process::exit(3);
    });
    (Some(fd), kind, kid)
  } else {
    (None, SocketKind::Raw, None)
  };

  let pid_id = (std::process::id() & 0xFFFF) as u16;
  let my_id = dgram_id4.or(dgram_id6).unwrap_or(pid_id);

  let interval = Duration::from_millis(args.interval);
  let period   = Duration::from_millis(args.period);
  let timeout  = Duration::from_millis(args.timeout);

  for (i, h) in hosts.iter_mut().enumerate() {
    h.next_send    = Instant::now() + interval * i as u32;
    h.retries_left = args.retry;
  }

  // seq -> PendingPing
  let mut seqmap: HashMap<u16, PendingPing> = HashMap::new();
  let mut seq_counter: u32 = 0;
  let mut recv_buf = vec![0u8; 4096];

  let start = Instant::now();
  let max_len = max_host_len(&hosts);

  // Start main_loop
  loop {
    let now = Instant::now();

    if hosts.iter().all(|h| h.done) && seqmap.is_empty() {
      break;
    }

    for idx in 0..hosts.len() {
      if hosts[idx].done || now < hosts[idx].next_send {
        continue;
      }

      let ping_idx = hosts[idx].current_ping_index;
      let seq: u16 = loop {
        let candidate = (seq_counter & 0xFFFF) as u16;
        seq_counter = seq_counter.wrapping_add(1);
        if !seqmap.contains_key(&candidate) {
          break candidate;
        }
        if (seq_counter & 0xFFFF) as u16 == (seq_counter.wrapping_sub(65536) & 0xFFFF) as u16 {
          break u16::MAX; // sentinel – seqmap.contains_key will still guard the insert below
        }
      };

      let is_ipv6  = hosts[idx].is_ipv6;
      let kind = if is_ipv6 { kind6 } else { kind4 };
      let pkt = build_icmp_packet(my_id, seq, args.size, is_ipv6, kind);

      let sent = match hosts[idx].addr {
        IpAddr::V4(ref a) => fd4.map(|fd| send_ping_v4(fd, a, &pkt)).unwrap_or(false),
        IpAddr::V6(ref a) => fd6.map(|fd| send_ping_v6(fd, a, &pkt)).unwrap_or(false),
      };

      if sent && !seqmap.contains_key(&seq) {
        let sent_at = Instant::now();
        seqmap.insert(seq, PendingPing { host_index: idx, ping_index: ping_idx, sent_at });
        hosts[idx].num_sent  += 1;
        hosts[idx].last_send  = Some(sent_at);

        hosts[idx].next_send = if count.is_some() || loop_mode {
          sent_at + period
        } else {
          let backoff = args.backoff.powi(hosts[idx].num_sent as i32 - 1);
          sent_at + timeout.mul_f64(backoff)
        };

        hosts[idx].current_ping_index += 1;

        if count.map(|c| hosts[idx].current_ping_index >= c).unwrap_or(false) {
          hosts[idx].next_send = now + Duration::from_secs(86400);
        }

        let is_default_mode = count.is_none() && !loop_mode;
        if is_default_mode && hosts[idx].current_ping_index > args.retry {
          hosts[idx].next_send = now + Duration::from_secs(86400);
        }
      }
    }

    for (fd_opt, is_v6, kind) in &[(fd4, false, kind4), (fd6, true, kind6)] {
      let fd = match fd_opt { Some(f) => *f, None => continue };
      loop {
        let received = match recv_ping(fd, &mut recv_buf, *is_v6, *kind) {
          Some(r) => r,
          None => break,
        };

        if received.id != my_id { continue; }

        if let Some(pending) = seqmap.get(&received.seq) {
          if Instant::now().duration_since(pending.sent_at) > timeout {
            seqmap.remove(&received.seq);
            continue;
          }
        }

        if let Some(pending) = seqmap.remove(&received.seq) {
          let rtt = Instant::now().duration_since(pending.sent_at);
          let hi  = pending.host_index;
          let first_reply = hosts[hi].num_recv == 0;
          hosts[hi].record_reply(rtt, pending.ping_index);

          let is_default_mode = count.is_none() && !loop_mode;
          if is_default_mode && first_reply {
            hosts[hi].done = true;
          }

          if !args.quiet && !args.unreach {
            if is_default_mode {
              if first_reply {
                print_alive(&hosts[hi], args.timestamp, args.json);
              }
            } else {
              print_recv(RecvLineOpts {
                host: &hosts[hi],
                ping_index: pending.ping_index,
                rtt,
                raw_len: received.raw_len,
                max_len,
                timestamp: args.timestamp,
                json: args.json,
                verbose_count,
              });
            }
          }
        }
      }
    }

    let now2 = Instant::now();
    let timed_out: Vec<u16> = seqmap
      .iter()
      .filter(|(_, p)| now2.duration_since(p.sent_at) > timeout)
      .map(|(&seq, _)| seq)
      .collect();

    for seq in timed_out {
      if let Some(pending) = seqmap.remove(&seq) {
        let is_default_mode = count.is_none() && !loop_mode;
        if !is_default_mode && !args.quiet && !args.alive {
          print_timeout(TimeoutLineOpts {
            host: &hosts[pending.host_index],
            ping_index: pending.ping_index,
            max_len,
            timestamp: args.timestamp,
            json: args.json,
          });
        }
      }
    }

    let now3 = Instant::now();
    for h in hosts.iter_mut() {
      if h.done { continue; }

      let last_expired = h.last_send
        .map(|s| now3.duration_since(s) > timeout)
        .unwrap_or(false);

      if let Some(c) = count {
        if h.current_ping_index >= c && last_expired {
          h.done = true;
        }
      } else if !loop_mode {
        if h.num_recv > 0 {
          h.done = true;
        } else if h.num_sent > args.retry && last_expired {
          h.done = true;
        }
      }
    }

    std::thread::sleep(Duration::from_millis(1));
  }

  let max_len = max_host_len(&hosts);
  let is_default_mode = count.is_none() && !loop_mode;

  if is_default_mode {
    if !args.quiet {
      for h in &hosts {
        if h.num_recv == 0 {
          print_unreachable(h, args.timestamp, args.json);
        }
      }
    }
  } else {
    for h in &hosts {
      if args.alive  && h.num_recv > 0 { println!("{}", h.display); }
      if args.unreach && h.num_recv == 0 { println!("{}", h.display); }
    }
  }

  if count.is_some() && !args.alive && !args.unreach {
    for h in &hosts {
      print_per_host_stats(h, max_len, args.json, verbose_count || args.report_all_rtts);
    }
  }

  if args.stats {
    let all_rtts: Vec<Duration> = hosts.iter()
      .flat_map(|h| h.resp_times.iter().filter_map(|r| *r))
      .collect();

    let g_sum: Duration = all_rtts.iter().sum();
    let g_count = all_rtts.len();

    print_global_stats(&GlobalStatsSummary {
      num_hosts:      hosts.len(),
      num_alive:      hosts.iter().filter(|h| h.num_recv > 0).count(),
      num_unreachable: hosts.iter().filter(|h| h.num_recv == 0).count(),
      total_sent:     hosts.iter().map(|h| h.num_sent).sum(),
      total_recv:     hosts.iter().map(|h| h.num_recv).sum(),
      min_rtt:        all_rtts.iter().min().copied(),
      avg_rtt:        (g_count > 0).then(|| g_sum / g_count as u32),
      max_rtt:        all_rtts.iter().max().copied(),
      elapsed:        start.elapsed(),
    }, args.json);
  }

  let exit_ok = if let Some(min_reach) = args.reachable {
    hosts.iter().filter(|h| h.num_recv > 0).count() as u32 >= min_reach
  } else {
    hosts.iter().all(|h| h.num_recv > 0)
  };

  if !exit_ok {
    std::process::exit(1);
  }
}