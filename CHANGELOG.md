# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Every member of the [oxml](https://github.com/sebastienrousseau/oxml)
suite ships the same version number.

## [0.0.11] - 2026-10-17

### Changed

- Pin Scorecard execution by verified container digest and disable public
  results publication. Retain SARIF artifacts, categorize posture findings
  in GitHub code scanning, and display workflow status instead of a score.

- Permit NCSA only for libfuzzer-sys 0.4.13 in the fuzz workspace;
  retain the root license policy and check both graphs in CI.
- Require an annotated tag signed by the approved public key, with the
  exact project/version subject, and use crates.io Trusted Publishing
  instead of a stored publishing token. Registry configuration still
  requires independent confirmation before tagging.

- Align unpublished harnesses at 0.0.11 and match example coverage to
  exact source paths, with negative regression contracts in local/CI gates.
- Wait for all actual release prerequisites before compiling, propagate
  publication failures and keep the laptop helper dry-run-only.

- Bound JSON sources at 16 MiB and arrays/objects at 256 nested levels.
- Reject malformed numeric spellings, unknown string escapes and raw
  control characters. Retain valid JSON and finite-number round trips.
- Make the laptop release helper dry-run-only and test that contract in CI.
- Align the publishable package and family dependencies at 0.0.11,
  refresh root/fuzz lockfiles, and retain Rust 1.89.0 as the minimum.
- Validate closed issue/dependency changes against current sources and
  use pinned compiler, coverage, fuzz and audit tools in CI.

- **Breaking: the MSRV is now Rust 1.89.0** (was 1.86.0), one floor
  shared with the maintainer's other Rust crates.
- **The MSRV job checks its toolchain first.** It already set
  `RUSTUP_TOOLCHAIN`; it now also asserts `rustc --version` before
  building.

## [0.0.10] - 2026-10-05

### Changed

- **Ecosystem release cycle v0.0.10.** Begin next iteration cycle across the
  entire lockstep suite.
- **Documentation migration.** Migrated website and rendered manual to
  `oxmllib.com` (`oxmllib.github.io`) using the Lucid SSG theme.

## [0.0.9] - 2026-10-04

### Changed

- **Version synchronization.** Synchronize suite release to 0.0.9 across
  all repositories.

## [0.0.8] - 2026-08-29

### Added

- Extracted from `oxml-mcp`, where this implementation lived, and from
  `oxml-lsp`, which had a copy of it. Neither could reach the other's
  private module, so a defect fixed in one would not have reached the
  other and nothing would have noticed.

### Fixed

- **A number outside `f64`'s range is refused rather than parsed to
  infinity.** `1e999` is grammatically valid JSON; parsing it yielded
  infinity, and infinity has no JSON representation, so serialising it
  again produced output that would not parse.

  The defect was in both copies. It surfaced here because the fuzz
  target moved with a stronger assertion — anything that parses must
  survive a round trip — and found it in ninety seconds on `-3e805`.

  Refusing is the honest answer for a parser that carries numbers as
  `f64`. Writing `null` instead, which some libraries do, silently
  changes the value.
