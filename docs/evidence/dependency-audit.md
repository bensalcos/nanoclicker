# Dependency audit

Date: 2026-10-24. Tool: cargo-audit 0.22.2 against the complete Cargo.lock. RustSec database updated 2026-10-23, commit 8a5b8090ef20c9c1b425f3373f28b0b23cc5c7a0. Structured summary: dependency-audit.json; full local output: artifacts/cargo-audit.json.

Result at that audit: **0 known vulnerabilities and 3 maintenance warnings**, rather than a warning-free result:

- RUSTSEC-2026-0371: zbus_names 4.3.5 is no longer maintained as a separate crate; integrated into zbus 6.
- RUSTSEC-2026-0372: zvariant_derive 5.15.1 is no longer maintained as a separate crate; macros integrated into zbus 6.
- RUSTSEC-2026-0370: zvariant 5.15.1 is no longer maintained as a separate crate; wire format integrated into zbus 6.

At the audit date, published versions were checked: ashpd 0.13.13, eframe 0.37.0 and accesskit_unix 0.24.0 were available and used the older D-Bus family. No incompatible semver substitution or advisory ignores were added. Review upstream migrations before distribution acceptance and keep these warnings visible.

Tool installation in this environment: the user's cc wrapper runs zig cc and failed on an aws-lc-sys --target argument. The failure was reproduced/localized, then cargo-audit was compiled using CC=/usr/bin/gcc, CXX=/usr/bin/g++ and /usr/bin/gcc as linker for that command only. No global compiler configuration or audit strength was changed. The tool is under /tmp/nanoclick-tools.
