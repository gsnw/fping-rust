/* cargo rust */
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const _PROGRAM: &str = env!("CARGO_PKG_NAME");

/* ICMP Typ */
pub const ICMP_ECHO_REQUEST: u8 = 8;
pub const ICMP_ECHO_REPLY: u8 = 0;
pub const ICMP6_ECHO_REQUEST: u8 = 128;
pub const ICMP6_ECHO_REPLY: u8 = 129;
pub const ICMP_HEADER_LEN: usize = 8;
pub const ICMP_TYPE_OFFSET: usize = 0;
pub const ICMP_ID_OFFSET: usize = 4;
pub const ICMP_SEQ_OFFSET: usize = 6;
pub const ICMP_MIN_LEN: usize = 8;
pub const IPV4_MIN_HDR_LEN: usize = 20;

/* Default values */
pub const _DEFAULT_INTERVAL_MS: u64 = 10;
pub const _DEFAULT_PERIOD_MS: u64 = 1000;
pub const _DEFAULT_TIMEOUT_MS: u64 = 500;
pub const _DEFAULT_RETRY: u32 = 3;
pub const _DEFAULT_BACKOFF: f64 = 1.5;
pub const _DEFAULT_PING_DATA_SIZE: usize = 56;

pub const _MIN_BACKOFF: f64 = 1.0;
pub const _MAX_BACKOFF: f64 = 5.0;

pub const _MAX_GENERATE: usize = 131_072;
pub const _MAX_TARGET_NAME: usize = 255;

/* Response flags */
pub const RESP_TIMES_CAP: usize = 1000;
pub const _RESP_WAITING: i64 = -1;
pub const _RESP_UNUSED: i64 = -2;
pub const _RESP_TIMEOUT: i64 = -4;

/* ICMP-Typ-Name */
pub const _ICMP_TYPE_STR: &[&str] = &[
  "ICMP Echo Reply",          // 0
  "",
  "",
  "ICMP Unreachable",         // 3
  "ICMP Source Quench",       // 4
  "ICMP Redirect",            // 5
  "",
  "",
  "ICMP Echo",                // 8
  "",
  "",
  "ICMP Time Exceeded",       // 11
  "ICMP Parameter Problem",   // 12
  "ICMP Timestamp Request",   // 13
  "ICMP Timestamp Reply",     // 14
  "ICMP Information Request", // 15
  "ICMP Information Reply",   // 16
  "ICMP Mask Request",        // 17
  "ICMP Mask Reply",          // 18
];

pub const _ICMP_UNREACH_STR: &[&str] = &[
  "ICMP Network Unreachable",
  "ICMP Host Unreachable",
  "ICMP Protocol Unreachable",
  "ICMP Port Unreachable",
  "ICMP Unreachable (Fragmentation Needed)",
  "ICMP Unreachable (Source Route Failed)",
  "ICMP Unreachable (Destination Network Unknown)",
  "ICMP Unreachable (Destination Host Unknown)",
  "ICMP Unreachable (Source Host Isolated)",
  "ICMP Unreachable (Communication with Network Prohibited)",
  "ICMP Unreachable (Communication with Host Prohibited)",
  "ICMP Unreachable (Network Unreachable For Type Of Service)",
  "ICMP Unreachable (Host Unreachable For Type Of Service)",
  "ICMP Unreachable (Communication Administratively Prohibited)",
  "ICMP Unreachable (Host Precedence Violation)",
  "ICMP Unreachable (Precedence cutoff in effect)",
];