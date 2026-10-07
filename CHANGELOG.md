# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [1.0.0] - 2026-10-07

### Added

- `chepstow auth` logs in to Isambard Keycloak with the OAuth 2.0 device flow. Choose dev or prod with `--env`.
- `chepstow token` prints a valid access token for LiteLLM, refreshing it when needed. It works as Claude Code's `apiKeyHelper`.
- `chepstow whoami` shows the token's claims, `chepstow models` lists the models LiteLLM offers, and `chepstow logout` revokes and deletes the cached tokens.
- Pre-built binaries for Linux (x86_64 and aarch64, glibc and musl), macOS (Apple silicon and Intel), Windows and FreeBSD.

[1.0.0]: https://github.com/isambard-sc/chepstow/releases/tag/1.0.0
