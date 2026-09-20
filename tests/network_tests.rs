use std::process::Command;
use regex::Regex;

#[test]
fn ping_localhost() {
  let output = Command::new(env!("CARGO_BIN_EXE_fping"))
    .args(["127.0.0.1"])
    .output()
    .expect("fping could not be started");

  assert!(
    output.status.success(),
    "fping terminated with an error: {:?}\nstdout:\n{}\nstderr:\n{}",
    output.status,
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("127.0.0.1 is alive"), "fping output does not include '127.0.0.1 is alive':\n{}", stdout);
}

#[test]
fn ping_count_one_with_regex() {
  let output = Command::new(env!("CARGO_BIN_EXE_fping"))
    .args(["-c", "1", "127.0.0.1"])
    .output()
    .expect("fping could not be started");

  assert!(
    output.status.success(),
    "fping terminated with an error: {:?}\nstdout:\n{}\nstderr:\n{}",
    output.status,
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  let resp_regex = Regex::new(r#"127\.0\.0\.1\s+:\s+\[0\]\s*,\s*\d+\s+bytes\s*,\s*[\d\.]+\s+ms\s+\([\d\.]+\s+avg\s*,\s*\d+%\s+loss\)"#).unwrap();
  assert!(
    resp_regex.is_match(&stdout),
    "Stdout did not match expected response pattern:\n{}",
    stdout
  );

  let stats_regex = Regex::new(r#"127\.0\.0\.1\s+:\s+xmt/rcv/%loss\s*=\s*1/1/0%,\s+min/avg/max\s*=\s*[\d\.]+/[\d\.]+/[\d\.]+"#).unwrap();
  assert!(
    stats_regex.is_match(&stderr),
    "Stderr did not match expected stats pattern:\n{}",
    stderr
  );
}

#[test]
fn ping_count_three_colon() {
  let output = Command::new(env!("CARGO_BIN_EXE_fping"))
    .args(["-C", "3", "127.0.0.1"])
    .output()
    .expect("fping could not be started");

  assert!(
    output.status.success(),
    "fping terminated with an error: {:?}\nstdout:\n{}\nstderr:\n{}",
    output.status,
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  let regex = Regex::new(r#"(?m)^127\.0\.0\.1\s+:\s+\[0\]\s*,\s*[\d\.]+\s+ms\s*\n127\.0\.0\.1\s+:\s+\[1\]\s*,\s*[\d\.]+\s+ms\s*\n127\.0\.0\.1\s+:\s+\[2\]\s*,\s*[\d\.]+\s+ms\s*$"#).unwrap();
  assert!(
    regex.is_match(&stdout),
    "Stdout did not match expected -C pattern:\n{}",
    stdout
  );
}

#[test]
fn ping_unreachable() {
  let output = Command::new(env!("CARGO_BIN_EXE_fping"))
    .args(["-u", "1.2.3.4"])
    .output()
    .expect("fping could not be started");

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(
    stdout.contains("1.2.3.4 is unreachable"),
    "fping output does not include '1.2.3.4 is unreachable':\n{}",
    stdout
  );
}

#[test]
fn ping_stats() {
  let output = Command::new(env!("CARGO_BIN_EXE_fping"))
    .args(["-s", "127.0.0.1"])
    .output()
    .expect("fping could not be started");

  assert!(
    output.status.success(),
    "fping terminated with an error: {:?}\nstdout:\n{}\nstderr:\n{}",
    output.status,
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr),
  );

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  assert!(
    stdout.contains("127.0.0.1 is alive") && stderr.contains("1 targets") && stderr.contains("min round trip time"),
    "fping output does not match expected stats format:\nstdout:\n{}\nstderr:\n{}",
    stdout,
    stderr
  );
}
