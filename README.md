# fping-rust

[<img alt="github" src="https://img.shields.io/badge/github-gsnw/fping-rust?style=for-the-badge&logo=github" height="20">](https://github.com/gsnw/fping-rust)
[![Rust CI](https://github.com/gsnw/fping-rust/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/gsnw/fping-rust/actions/workflows/rust-ci.yml)

fping-rust is an attempt to translate the fping program into Rust.

This version is licensed under the GPL

## Original

Original fping is a program to send ICMP echo probes to network hosts, similar to ping,
but much better performing when pinging multiple hosts. fping has a long long
story: Roland Schemers did publish a first version of it in 1992 and it has
established itself since then as a standard tool.

## Installation

```
cargo build
cargo install --path=/usr/local
```

Make fping either setuid, or, if under Linux

```
sudo setcap cap_net_raw,cap_net_admin+ep fping`
```

If you can't run fping as root or can't use the cap_net_raw capability, you can also run fping in unprivileged mode. This works on MacOS and also on Linux, provided that your GID is included in the range defined in `/proc/sys/net/ipv4/ping_group_range`. This is particularly useful for running fping-rust in rootless / unprivileged containers.

```
echo "net.ipv4.ping_group_range = 0 2147483647" >> /etc/sysctl.d/local.conf
```

## Usage

```
Fast ping to multiple hosts

Usage: fping [OPTIONS] [TARGETS]...

Arguments:
  [TARGETS]...  Target hosts

Options:
  -c, --count <N>          Count mode: send N pings to each target
  -C, --vcount <N>         Same as -c but verbose output (all RTTs)
  -l, --loop               Loop mode: send pings forever
  -i, --interval <MSEC>    Interval between packets in ms (default: 10) [default: 10]
  -p, --period <MSEC>      Per-host interval in ms (default: 1000) [default: 1000]
  -t, --timeout <MSEC>     Timeout in ms (default: 500) [default: 500]
  -r, --retry <RETRY>      Number of retries (default: 3) [default: 3]
  -B, --backoff <BACKOFF>  Exponential backoff factor (default: 1.5) [default: 1.5]
  -b, --size <BYTES>       Ping data size in bytes (default: 56) [default: 56]
  -f, --file <FILE>        Read hosts from file (- = stdin)
  -a, --alive              Show only alive hosts
  -u, --unreach            Show only unreachable hosts
  -q, --quiet              Quiet: don't show per-ping results
  -s, --stats              Print final stats
  -e, --elapsed            Show elapsed time on received packets
  -A, --addr               Show targets by address
  -D, --timestamp          Timestamp before each line
  -J, --json               JSON output (requires -c, -C or -l)
  -4, --ipv4               Use IPv4 only
  -6, --ipv6               Use IPv6 only
      --report-all-rtts    Show all individual RTTs
  -x, --reachable <N>      Minimum number of reachable hosts to be considered success
  -h, --help               Print help
  -V, --version            Print version
```

## Test

### Run all tests

```
cargo test
```

### Run special test

```
cargo test --test args_tests
cargo test --test dns_tests
cargo test --test output_tests
cargo test --test socket_tests
cargo test --test types_tests
```

### Run specific test by name

```
cargo test --test types_tests loss_pct_50
```

### Overview of the test files

| File | Description | root required |
| ---- | ----------- | ------------- |
| args_tests.rs | CLI argument parsing, default values | No |
| dns_tests.rs | Hostname resolution, IP filtering | No* |
| output_tests.rs | `sprint_tm`, `max_host_len` | No |
| socket_tests.rs | Packet construction, checksum, ID/Seq encoding | No |
| types_tests.rs | `HostEntry` logic, statistics | No |

&#42; Network access is required for DNS resolution (localhost is always available).

## Reference

* https://github.com/schweikert/fping