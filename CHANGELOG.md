fping-rust 0.1.1 (2026-03-28)
======================
- fix: validate ICMP ID per socket, fix dual-stack ID mismatch (#15, @gsnw-sebast)
- fix: close raw sockets via OwnedFd to prevent fd leak (#14, @gsnw-sebast)
- fix: record RTTs in loop/default mode for correct -s statistics (#13, @gsnw-sebast)
- fix: Sequence number collisions (#12, @gsnw-sebast)
- fix: remove spurious .to_be() in send_ping_v4 (#10, @ijohanne)

fping-rust 0.1.0 (2026-03-19)
======================
- Update cargo metadata and github action release
- Add metadata description and license
- Add github action codeQL
- Remove create changelog from release pipline and add a ci script
- Update github release pipline
- Resolves a warning regarding unused constants and variables and removes a structure
- Update to the README
- Create github action release for crates.io
- Add GPLv3 license
- Remove windows-latest from github workflow and limit push only on main
- Fix macOS mismatched types
- Create github rust-ci workflow
- Fix dns resolve fallback if ipv6 not found
- Unit-Tests created
- Fix SOCK_DGRAM for unprivileged users
- Fix default mode without root
- First version of fping-rust