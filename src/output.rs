use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::types::HostEntry;

pub fn sprint_tm(d: Duration) -> String {
  let ms = d.as_secs_f64() * 1000.0;
  if ms < 1.0 {
    format!("{:.3}", ms)
  } else if ms < 10.0 {
    format!("{:.2}", ms)
  } else if ms < 100.0 {
    format!("{:.1}", ms)
  } else if ms < 1_000_000.0 {
    format!("{:.0}", ms)
  } else {
    format!("{:.3e}", ms)
  }
}

pub fn now_ts() -> String {
  let secs = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_secs_f64();
  format!("[{:.5}]", secs)
}

pub fn max_host_len(hosts: &[HostEntry]) -> usize {
  hosts.iter().map(|h| h.display.len()).max().unwrap_or(0)
}

pub fn print_alive(host: &HostEntry, timestamp: bool, json: bool) {
  let prefix = if timestamp { format!("{} ", now_ts()) } else { String::new() };
  if json {
    println!("{{\"alive\": {{\"host\": \"{}\"}}}}", host.display);
  } else {
    println!("{}{} is alive", prefix, host.display);
  }
}

pub fn print_unreachable(host: &HostEntry, timestamp: bool, json: bool) {
  let prefix = if timestamp { format!("{} ", now_ts()) } else { String::new() };
  if json {
    println!("{{\"unreachable\": {{\"host\": \"{}\"}}}}", host.display);
  } else {
    println!("{}{} is unreachable", prefix, host.display);
  }
}

pub struct RecvLineOpts<'a> {
  pub host: &'a HostEntry,
  pub ping_index: u32,
  pub rtt: Duration,
  pub raw_len: usize,
  pub max_len: usize,
  pub timestamp: bool,
  pub json: bool,
  pub verbose_count: bool,
}

pub fn print_recv(opts: RecvLineOpts) {
  let prefix = if opts.timestamp {
    format!("{} ", now_ts())
  } else {
    String::new()
  };

  let h = opts.host;
  let rtt_str = sprint_tm(opts.rtt);

  if opts.json {
    println!(
      "{{\"resp\": {{\"host\": \"{}\", \"seq\": {}, \"rtt\": {}}}}}",
      h.display, opts.ping_index, rtt_str
    );
    return;
  }

  if opts.verbose_count {
    println!(
      "{}{:<width$} : [{}], {} ms",
      prefix, h.display, opts.ping_index, rtt_str,
      width = opts.max_len
    );
    return;
  }

  let avg_str = h.avg_reply().map(sprint_tm).unwrap_or_default();
  println!(
    "{}{:<width$} : [{}], {} bytes, {} ms ({} avg, {}% loss)",
    prefix, h.display, opts.ping_index,
    opts.raw_len.saturating_sub(28),
    rtt_str, avg_str, h.loss_pct(),
    width = opts.max_len
  );
}

pub struct TimeoutLineOpts<'a> {
  pub host: &'a HostEntry,
  pub ping_index: u32,
  pub max_len: usize,
  pub timestamp: bool,
  pub json: bool,
}

pub fn print_timeout(opts: TimeoutLineOpts) {
  let prefix = if opts.timestamp {
    format!("{} ", now_ts())
  } else {
    String::new()
  };

  let h = opts.host;

  if opts.json {
    println!(
      "{{\"timeout\": {{\"host\": \"{}\", \"seq\": {}}}}}",
      h.display, opts.ping_index
    );
    return;
  }

  print!(
    "{}{:<width$} : [{}], timed out",
    prefix, h.display, opts.ping_index,
    width = opts.max_len
  );

  match h.avg_reply() {
    Some(avg) => print!(" ({} avg, ", sprint_tm(avg)),
    None => print!(" (NaN avg, "),
  }

  if h.num_recv <= h.num_sent {
    println!("{}% loss)", h.loss_pct());
  } else {
    println!(
      "{}% return)",
      (h.num_recv * 100) / h.num_sent.max(1)
    );
  }
}

pub fn print_per_host_stats(host: &HostEntry, max_len: usize, json: bool, verbose: bool) {
  if verbose {
    let rtts: Vec<String> = host.resp_times
      .iter()
      .map(|r| r.as_ref().map(|d| sprint_tm(*d)).unwrap_or_else(|| "-".into()))
      .collect();

    if json {
      let vals: Vec<String> = host.resp_times
        .iter()
        .map(|r| r.as_ref().map(|d| sprint_tm(*d)).unwrap_or_else(|| "null".into()))
        .collect();
      eprintln!(
        "{{\"vSum\": {{\"host\": \"{}\", \"values\": [{}]}}}}",
        host.display,
        vals.join(", ")
      );
    } else {
      eprint!("{:<width$} :", host.display, width = max_len);
      for rtt in &rtts {
        eprint!(" {}", rtt);
      }
      eprintln!();
    }
    return;
  }

  if json {
    eprint!(
      "{{\"summary\": {{\"host\": \"{}\", \"xmt\": {}, \"rcv\": {}, \"loss\": {}",
      host.display, host.num_sent, host.num_recv, host.loss_pct()
    );
    if let (Some(mn), Some(av), Some(mx)) =
      (host.min_reply, host.avg_reply(), host.max_reply)
    {
      eprint!(
        ", \"rttMin\": {}, \"rttAvg\": {}, \"rttMax\": {}",
        sprint_tm(mn), sprint_tm(av), sprint_tm(mx)
      );
    }
    eprintln!("}}}}");
  } else {
    eprint!(
      "{:<width$} : xmt/rcv/%loss = {}/{}/{}%",
      host.display, host.num_sent, host.num_recv, host.loss_pct(),
      width = max_len
    );
    if let (Some(mn), Some(av), Some(mx)) =
      (host.min_reply, host.avg_reply(), host.max_reply)
    {
      eprint!(
        ", min/avg/max = {}/{}/{}",
        sprint_tm(mn), sprint_tm(av), sprint_tm(mx)
      );
    }
    eprintln!();
  }
}

pub struct GlobalStatsSummary {
  pub num_hosts: usize,
  pub num_alive: usize,
  pub num_unreachable: usize,
  pub total_sent: u32,
  pub total_recv: u32,
  pub min_rtt: Option<Duration>,
  pub avg_rtt: Option<Duration>,
  pub max_rtt: Option<Duration>,
  pub elapsed: Duration,
}

pub fn print_global_stats(s: &GlobalStatsSummary, json: bool) {
  if json {
    println!(
      "{{\"stats\": {{\"targets\": {}, \"alive\": {}, \"unreachable\": {}, \
        \"icmpEchosSent\": {}, \"icmpEchoRepliesReceived\": {}, \"elapsed\": {:.3}}}}}",
      s.num_hosts, s.num_alive, s.num_unreachable,
      s.total_sent, s.total_recv,
      s.elapsed.as_secs_f64()
    );
    return;
  }

  eprintln!();
  eprintln!(" {:>7} targets",       s.num_hosts);
  eprintln!(" {:>7} alive",         s.num_alive);
  eprintln!(" {:>7} unreachable",   s.num_unreachable);
  eprintln!();
  eprintln!(" {:>7} ICMP Echos sent",            s.total_sent);
  eprintln!(" {:>7} ICMP Echo Replies received", s.total_recv);
  eprintln!();

  if let (Some(mn), Some(av), Some(mx)) = (s.min_rtt, s.avg_rtt, s.max_rtt) {
    eprintln!(" {} ms (min round trip time)", sprint_tm(mn));
    eprintln!(" {} ms (avg round trip time)", sprint_tm(av));
    eprintln!(" {} ms (max round trip time)", sprint_tm(mx));
  }

  eprintln!(" {:>12.3} sec (elapsed real time)", s.elapsed.as_secs_f64());
  eprintln!();
}