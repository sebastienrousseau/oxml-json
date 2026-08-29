# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Every member of the [oxml](https://github.com/sebastienrousseau/oxml)
suite ships the same version number.

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
