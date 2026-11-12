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

Libee is organized **feature-first**: each feature owns its model, logic and Slint view.

```
src/
├── main.rs        # entry point, wires features to the window
├── shared/        # errors, structured commands, common types
├── managers/      # one adapter per package manager (brew, npm…)
└── features/
    ├── inventory/ # model.rs, scan.rs, view.slint
    ├── cleanup/   # preview.rs, run.rs, view.slint
    └── history/
ui/
├── main.slint     # app window and navigation
└── theme/         # design tokens and reusable components
```

Folders are created when the first feature needs them — no empty placeholders.

## Safety rules

- Never build commands by string concatenation or through a shell.
- Every destructive action goes through a preview and an explicit confirmation.
- A failing package manager must never block the others.
