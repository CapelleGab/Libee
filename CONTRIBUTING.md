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

Rust logic lives in `src/`, the Slint interface lives in `ui/`.

```
src/
├── main.rs
├── app/          # the only bridge between Rust and Slint: state, callbacks, navigation
├── models/       # Tool, Manager, Scope, Status…
├── managers/     # one adapter per package manager (brew, npm…)
├── services/     # inventory scan, cleanup, history
└── shared/       # errors, structured commands, utilities

ui/
├── app.slint     # root window and routing
├── theme/        # design tokens: colors, typography, spacing, radius
├── components/
│   ├── ui/       # primitives: button, badge, card, input
│   └── …         # composites: sidebar, tool row, stat card
├── views/        # screens: inventory, tool detail, history
└── assets/       # logo, fonts, icons
```

- `ui/` holds no logic: data comes in through properties, actions go out through callbacks.
- `models`, `managers` and `services` never depend on Slint, so they stay testable.
- Folders are created when the first piece of code needs them.

## Safety rules

- Never build commands by string concatenation or through a shell.
- Every destructive action goes through a preview and an explicit confirmation.
- A failing package manager must never block the others.
