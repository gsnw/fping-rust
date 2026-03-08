/*
 * fping-rs – Rust reimplementation of fping
 */

mod args;
mod constants;
mod dns;
mod output;
mod pinger;
mod socket;
mod types;

use clap::Parser;
use std::fs::File;
use std::io::{self, BufRead};

use args::Args;
use dns::resolve;

fn main() {
  let args = Args::parse();

  let mut host_names: Vec<String> = Vec::new();

  if let Some(ref path) = args.file {
    let reader: Box<dyn BufRead> = if path == "-" {
      Box::new(io::BufReader::new(io::stdin()))
    } else {
      match File::open(path) {
        Ok(f) => Box::new(io::BufReader::new(f)),
        Err(e) => {
          eprintln!("fping: Cannot open ‘{}’: {}", path, e);
          std::process::exit(1);
        }
      }
    };
    for line in reader.lines().flatten() {
      let t = line.trim().to_string();
      if !t.is_empty() && !t.starts_with('#') {
        host_names.push(t);
      }
    }
  }

  host_names.extend(args.targets.iter().cloned());

  if host_names.is_empty() {
    eprintln!("fping: No targets specified.");
    eprintln!("Usage: fping [options] [hosts...]");
    std::process::exit(1);
  }

  let mut resolved: Vec<(String, std::net::IpAddr)> = Vec::new();
  for name in &host_names {
    match resolve(name, args.ipv4, args.ipv6) {
      Some(addr) => resolved.push((name.clone(), addr)),
      None => eprintln!("fping: Cannot resolve '{}'", name),
    }
  }

  if resolved.is_empty() {
    eprintln!("fping: No valid targets.");
    std::process::exit(1);
  }

  pinger::run(args, resolved);
}
