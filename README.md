# Warp Fork

`jupiternaut/warp` fork and mirror of the upstream Warp repository.

This checkout tracks the `master` branch. Do not assume a `main` branch when cloning, scripting, or handing this repository to another agent. The canonical upstream project is [`warpdotdev/warp`](https://github.com/warpdotdev/warp); this fork may carry mirror, handoff, or local branch context.

## Table of Contents

- [Background](#background)
- [Install](#install)
- [Usage](#usage)
- [Repository Layout](#repository-layout)
- [Status](#status)
- [Maintainer](#maintainer)
- [Contributing](#contributing)
- [License and Upstream](#license-and-upstream)

## Background

Warp is an agentic development environment born out of the terminal. This repository is a fork of the upstream open-source Warp client codebase. It includes the Rust application workspace, custom UI framework crates, assets, scripts, workflows, and contributor docs.

Fork boundary:

- Upstream: `warpdotdev/warp`.
- Local fork: `jupiternaut/warp`.
- Local branch for this handoff: `master`.
- Product downloads, support, and official docs still belong to Warp upstream unless a local fork change says otherwise.

## Install

For normal product use, download Warp from the official Warp site:

```text
https://www.warp.dev/download
```

For source development in this fork:

```sh
git clone --branch master https://github.com/jupiternaut/warp.git
cd warp
./script/bootstrap
```

Platform prerequisites are maintained by upstream scripts and docs. The bootstrap path may install platform build dependencies and common agent skills depending on options and environment.

## Usage

Build and run locally:

```sh
./script/run
```

Common engineering checks:

```sh
./script/presubmit
cargo fmt
cargo clippy --workspace --all-targets --all-features --tests -- -D warnings
```

For more engineering details, read [WARP.md](WARP.md) and [CONTRIBUTING.md](CONTRIBUTING.md).

## Repository Layout

- `app/` - main Warp application, terminal, AI, workspace, auth, settings, assets, and platform code.
- `crates/` - Rust workspace crates, including UI, editor, persistence, GraphQL, terminal, and platform libraries.
- `command-signatures-v2/` - command signature support.
- `script/` - bootstrap, run, presubmit, formatting, and build helper scripts.
- `.agents/`, `.claude/`, `.warp/` - agent skills, workflows, and local automation material.
- `.github/` - GitHub workflows, templates, and actions.
- `specs/`, `resources/`, `images/`, `docker/` - supporting project resources.
- `WARP.md` - engineering guide for agents and contributors working in the repo.
- `CONTRIBUTING.md` and `CODE_OF_CONDUCT.md` - upstream contribution and conduct rules.

## Status

Fork/mirror. The branch context is `master`.

Use upstream docs for official product claims. Use this fork for local development, branch-specific experiments, or handoff work that explicitly targets `jupiternaut/warp`.

## Maintainer

Maintained in the `jupiternaut/warp` fork.

## Contributing

For upstreamable work, follow [CONTRIBUTING.md](CONTRIBUTING.md), run presubmit checks, and target the correct upstream or fork branch. For local fork work, state whether the change is intended for:

- upstream `warpdotdev/warp`,
- fork-only handoff context,
- local automation or agent workflow support.

Always name `master` explicitly in scripts or handoffs that depend on this fork branch.

## License and Upstream

Warp is dual-licensed by area:

- `warpui_core` and `warpui` crates: MIT, see [LICENSE-MIT](LICENSE-MIT).
- Remaining repository code: AGPL-3.0, see [LICENSE-AGPL](LICENSE-AGPL).

Upstream: [`warpdotdev/warp`](https://github.com/warpdotdev/warp).

Local fork: [`jupiternaut/warp`](https://github.com/jupiternaut/warp), branch `master`.
