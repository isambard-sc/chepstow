# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.3.1] - 2026-10-16

### Changed

- Shortened the README to install, `chepstow auth` and `chepstow token`. It no longer covers the other subcommands, the cache location, or setup for opencode and Claude Code.

## [0.3.0] - 2026-10-12

### Changed

- Removed the `--issuer`, `--client-id`, `--base-url` and `--scope` options. They now come from `--env`, which is hidden from `--help`.
- `--env` now defaults to `prod`.
- `chepstow whoami` shows `selected_group` instead of `client_role`.
- Simplified docs.

## [0.2.0] - 2026-10-07

### Added

- `chepstow auth` logs in to Isambard Keycloak with the OAuth 2.0 device flow. Choose dev or prod with `--env`.
- `chepstow token` prints a valid access token for the inference service, refreshing it when needed. It works as Claude Code's `apiKeyHelper`.
- `chepstow whoami` shows the token's claims, `chepstow models` lists the models the inference service offers, and `chepstow logout` revokes and deletes the cached tokens.
- Pre-built binaries for Linux (x86_64 and aarch64, glibc and musl), macOS (Apple silicon and Intel), Windows and FreeBSD.

[0.3.1]: https://github.com/isambard-sc/chepstow/releases/tag/0.3.1
[0.3.0]: https://github.com/isambard-sc/chepstow/releases/tag/0.3.0
[0.2.0]: https://github.com/isambard-sc/chepstow/releases/tag/0.2.0
