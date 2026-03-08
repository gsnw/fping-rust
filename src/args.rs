use clap::Parser;
use crate::constants::*;

#[derive(Parser, Debug)]
#[command(
  name = "fping",
  version = VERSION,
  about = "Fast ping to multiple hosts")]

pub struct Args {
  /// Target hosts
  pub targets: Vec<String>,

  /// Count mode: send N pings to each target
  #[arg(short = 'c', long, value_name = "N")]
  pub count: Option<u32>,

  /// Same as -c but verbose output (all RTTs)
  #[arg(short = 'C', long = "vcount", value_name = "N")]
  pub vcount: Option<u32>,

  /// Loop mode: send pings forever
  #[arg(short = 'l', long)]
  pub r#loop: bool,

  /// Interval between packets in ms (default: 10)
  #[arg(short = 'i', long, value_name = "MSEC", default_value = "10")]
  pub interval: u64,

  /// Per-host interval in ms (default: 1000)
  #[arg(short = 'p', long, value_name = "MSEC", default_value = "1000")]
  pub period: u64,

  /// Timeout in ms (default: 500)
  #[arg(short = 't', long, value_name = "MSEC", default_value = "500")]
  pub timeout: u64,

  /// Number of retries (default: 3)
  #[arg(short = 'r', long, default_value = "3")]
  pub retry: u32,

  /// Exponential backoff factor (default: 1.5)
  #[arg(short = 'B', long, default_value = "1.5")]
  pub backoff: f64,

  /// Ping data size in bytes (default: 56)
  #[arg(short = 'b', long = "size", value_name = "BYTES", default_value = "56")]
  pub size: usize,

  /// Read hosts from file (- = stdin)
  #[arg(short = 'f', long, value_name = "FILE")]
  pub file: Option<String>,

  /// Show only alive hosts
  #[arg(short = 'a', long)]
  pub alive: bool,

  /// Show only unreachable hosts
  #[arg(short = 'u', long)]
  pub unreach: bool,

  /// Quiet: don't show per-ping results
  #[arg(short = 'q', long)]
  pub quiet: bool,

  /// Print final stats
  #[arg(short = 's', long)]
  pub stats: bool,

  /// Show elapsed time on received packets
  #[arg(short = 'e', long)]
  pub elapsed: bool,

  /// Show targets by address
  #[arg(short = 'A', long)]
  pub addr: bool,

  /// Timestamp before each line
  #[arg(short = 'D', long)]
  pub timestamp: bool,

  /// JSON output (requires -c, -C or -l)
  #[arg(short = 'J', long)]
  pub json: bool,

  /// Use IPv4 only
  #[arg(short = '4', long)]
  pub ipv4: bool,

  /// Use IPv6 only
  #[arg(short = '6', long)]
  pub ipv6: bool,

  /// Show all individual RTTs
  #[arg(long = "report-all-rtts")]
  pub report_all_rtts: bool,

  /// Minimum number of reachable hosts to be considered success
  #[arg(short = 'x', long = "reachable", value_name = "N")]
  pub reachable: Option<u32>,
}

impl Args {
  pub fn effective_count(&self) -> Option<u32> {
    self.vcount.or(self.count)
  }

  /// Ist verbose-count aktiv?
  pub fn is_verbose_count(&self) -> bool {
    self.vcount.is_some()
  }
}