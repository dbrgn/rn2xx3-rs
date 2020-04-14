# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.


### v0.1.4 (2020-04-15)

- [added] Optional logging, enabled through Cargo feature (#7)
- [added] Support for sleeping (#9)
- [added] Support for setting and querying data rate (#16)
- [added] Support for setting and querying ADR (#18)
- [added] Allow destroying driver instance to reclaim serial peripheral (#9)
- [changed] Expose error module (#8)
- [fixed] Don't generate getters for MAC keys (#15)

### v0.1.3 (2019-12-25)

- [fixed] Fix `no_std` compatibility

### v0.1.2 (2019-12-24)

- [added] Make this crate `no_std` compatible

### v0.1.1 (2019-12-06)

Only a metadata update (to ensure that the README gets rendered on crates.io).

### v0.1.0 (2019-12-06)

First crates.io release to reserve the name. What works so far: Basic
communication and reading the HWEUI.
