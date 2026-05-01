# Contributing to glctl

Thanks for your interest in glctl. This document explains how to file issues, propose changes, and ship a PR.

## Quick start for contributors

```sh
git clone https://github.com/baryonlabs/glctl.git
cd glctl
cargo build --release
./target/release/glctl --help
```

If `cargo --help` works and the binary runs, you're set.

Rust toolchain pinned in `.tool-versions` (asdf format). Stable Rust ≥ 1.74 should work without it.

## What kinds of contributions are welcome

- **Bug reports** — open an issue with the `bug` template. Include the exact command + glctl version + OS.
- **New subcommands** that fit the *Git-like local CLI* model (record / inspect / push generation history)
- **YAML schema additions** — coordinate with `baryonlabs/glhub` first; the schema is a contract between the two
- **Performance** for large lineages (>10K generations)
- **Cross-platform support** — Windows path issues, ARM64 macOS, statically-linked builds
- **Docs / examples** — especially anything that improves the 5-minute quickstart in the README

## What we'd rather you discuss first

Before significant effort, please open an issue:

- New top-level CLI verbs (current set: `init`, `new`, `list`, `lineage`, `graph`, `show`, `status`, `push`, `fsck`)
- Schema-breaking changes to the Generation YAML
- Auth / signing / encryption — these likely belong in the [Enterprise tier](https://github.com/baryonlabs/glhub#whats-oss-whats-enterprise)
- New network protocols (we currently push HTTP/JSON to glhub; alternatives need design)

## How to write a good PR

1. **Small** — one verb, one concern.
2. **Tested** — Rust integration tests in `tests/` are best; unit tests in `src/` second-best.
3. **Verified by command** — paste the actual `glctl` command output before/after in the PR description.
4. **`cargo fmt` + `cargo clippy --no-deps -- -D warnings`** before push.

## Commit messages

Conventional Commits, loose:

```
type(scope): subject

Body if needed.

Closes #N
```

Common types: `feat`, `fix`, `docs`, `chore`, `refactor`, `perf`, `test`.

## Code of conduct

Participating in this project means agreeing to follow our [Code of Conduct](./CODE_OF_CONDUCT.md).

## License

By contributing, you agree your contributions are licensed under the Apache License 2.0 (see [LICENSE](./LICENSE)). The previous `Cargo.toml` `license = "MIT"` has been corrected to match this LICENSE file.

## Companion repository

The matching hub server is at [`baryonlabs/glhub`](https://github.com/baryonlabs/glhub). Schema and HTTP API contract are shared. If your change touches the wire format, expect a coordinated PR pair.
