# fping-rust

fping-rust is an attempt to translate the fping program from https://github.com/schweikert/fping into Rust.

This version is licensed under the GPL

## Original

fping is a program to send ICMP echo probes to network hosts, similar to ping,
but much better performing when pinging multiple hosts. fping has a long long
story: Roland Schemers did publish a first version of it in 1992 and it has
established itself since then as a standard tool.

## Installation

```
cargo build
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