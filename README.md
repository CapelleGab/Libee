<div align="center">
  <img src="docs/assets/logo.svg" alt="Libee logo" width="128" />

  <h1>Libee</h1>

  <p><strong>See, understand and clean up the dev tools living on your machine.</strong></p>

  <p>
    <a href="https://github.com/CapelleGab/Libee/actions/workflows/ci.yml"><img src="https://github.com/CapelleGab/Libee/actions/workflows/ci.yml/badge.svg?branch=dev" alt="CI" /></a>
    <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-f59e0b" alt="License: GPLv3" /></a>
    <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Slint-1a1612" alt="Built with Rust + Slint" />
  </p>
</div>

---

You install a tool because a tutorial, a project or a colleague asked you to. The need goes away, the tool stays — along with its caches, old versions and forgotten dependencies.

**Libee** gives you a single, clear map of your local development environment: what is installed, by which package manager, where it lives, how much space it takes — and what can be safely removed.

## ✨ Principles

- 🔍 **Visibility before action** — Libee explains what it found before suggesting anything.
- 🛡️ **Trust and reversibility** — every destructive action is previewed, confirmed and logged.
- 🤝 **Respect what exists** — Libee drives your package managers instead of reinventing them.
- 🏠 **Local-first** — your inventory never leaves your machine.

## 📦 Supported package managers

| Platform | Managers | Status |
| --- | --- | --- |
| macOS | Homebrew, npm, pnpm, yarn | 🚧 MVP |
| Linux | apt, dnf, pacman, npm, pnpm, yarn | 🗓️ Planned |
| Windows | winget, Chocolatey, Scoop, npm, pnpm, yarn | 🗓️ Planned |

## 🚀 Getting started

```sh
git clone https://github.com/CapelleGab/Libee.git
cd Libee
cargo run
```

Requires a stable [Rust](https://rustup.rs) toolchain. See [CONTRIBUTING.md](CONTRIBUTING.md) for platform-specific dependencies.

## 🗺️ Roadmap

- [ ] **v0.1 — Foundations**: repository tooling, code structure, design system
- [ ] **v0.2 — Read-only inventory**: detect managers, unified and filterable view
- [ ] **v0.3 — Safe actions**: uninstall preview, execution, action log
- [ ] **v0.4 — Caches & recommendations**: caches, duplicates, old versions

Follow progress on the [milestones](https://github.com/CapelleGab/Libee/milestones).

## 🤝 Contributing

Contributions are welcome! Read the [contributing guide](CONTRIBUTING.md) and pick a [`good first issue`](https://github.com/CapelleGab/Libee/labels/good%20first%20issue).

## 📄 License

Libee is free software released under the [GNU GPLv3](LICENSE).
