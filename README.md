# fping-rust

fping-rust is an attempt to translate the fping program from https://github.com/schweikert/fping into Rust.

fping is a program to send ICMP echo probes to network hosts, similar to ping,
but much better performing when pinging multiple hosts. fping has a long long
story: Roland Schemers did publish a first version of it in 1992 and it has
established itself since then as a standard tool.

## Installation

```
cargo build
```