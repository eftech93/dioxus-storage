# Contributing to dioxus-storage

Thank you for your interest in contributing! This document provides guidelines for getting started.

## Development Environment

- **Rust**: >= 1.75
- **cargo**: latest stable

## Setup

```bash
git clone https://github.com/eftech93/dioxus-storage.git
cd dioxus-storage
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

## Branch Naming

- `feature/` — New features or enhancements
- `fix/` — Bug fixes
- `docs/` — Documentation improvements
- `chore/` — Maintenance tasks

## Pull Request Checklist

- [ ] Tests pass (`cargo test --all-features`)
- [ ] Clippy lints pass (`cargo clippy --all-targets --all-features -- -D warnings`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] New features include tests
- [ ] Documentation is updated (README, docs/)
- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/)

## Commit Message Format

```
type(scope): subject

body (optional)
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Questions?

Open a [Discussion](https://github.com/eftech93/dioxus-storage/discussions) or reach out to [eftech93@gmail.com](mailto:eftech93@gmail.com).
