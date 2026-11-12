# Contributing to Libee 🐝

Thanks for wanting to join the hive! This guide explains how we work.

## Prerequisites

- [Rust](https://rustup.rs) stable (edition 2024)
- On Linux: `libfontconfig1-dev` and `libxkbcommon-dev`

```sh
git clone https://github.com/CapelleGab/Libee.git
cd Libee
cargo run
```

## Git workflow

| Branch | Purpose |
| --- | --- |
| `main` | released versions, always stable |
| `dev` | integration branch for upcoming work |
| `feat/*`, `fix/*`, `chore/*`, `refactor/*`, `docs/*` | one branch per issue, created from `dev` |

1. Pick an issue (`good first issue` ones are great to start with).
2. Branch off `dev`: `git checkout -b feat/npm-adapter dev`.
3. Open a PR against `dev` and link the issue (`Closes #12`).

## Commits

Short [Conventional Commits](https://www.conventionalcommits.org):

```
type(scope): lowercase message
```

Types: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`, `ci`.
Keep commits **small and focused** (2–3 files max).

## Quality

Before pushing:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Architecture

```
src/
├── app/        # app state and orchestration, bridge to the UI
├── core/       # domain model (Tool, Manager…)
├── managers/   # one adapter per package manager
├── inventory/  # scan engine
└── platform/   # OS specifics
ui/
├── theme/      # design tokens (colors, type, spacing)
├── components/ # reusable components
└── views/      # screens
```

## Safety rules

- Never build commands by string concatenation or through a shell.
- Every destructive action goes through a preview and an explicit confirmation.
- A failing package manager must never block the others.
