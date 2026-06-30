# Security Policy

## Reporting Vulnerabilities

If you discover a security vulnerability in MarkQL, please report it responsibly by opening an issue or contacting the maintainer directly.

## Scope

MarkQL is a local CLI tool that parses Markdown files and runs queries against them. It does not:
- Make network requests
- Execute arbitrary code
- Access the filesystem beyond input files
- Store or transmit user data

## Input Validation

- All query syntax is validated before execution
- Invalid queries produce clear error messages, not crashes
- The Markdown parser handles malformed input gracefully

## Dependencies

Dependencies are pinned to specific versions in `Cargo.lock`. Run `cargo update` regularly to pull in security patches.

## Build

Always build from source or use trusted package registries:
```bash
cargo install markql
```
