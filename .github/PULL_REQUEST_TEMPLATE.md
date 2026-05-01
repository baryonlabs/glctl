## What changed

<!-- One sentence then a bullet list. The diff should make sense after reading this. -->

## Why

<!-- Briefly: which user / use case benefits, or which bug this fixes. Link an issue if it exists. -->

## How to verify

<!--
A reviewer should be able to follow these steps and see the change works.
Examples:
  - cargo build --release
  - ./target/release/glctl <new subcommand> --help
  - cargo test test_name
-->

```sh
# paste the actual command + expected output
```

## OSS / Cloud / Enterprise tier

- [ ] Stays in the OSS tier (CLI lineage primitives — record / inspect / push)
- [ ] Crosses into Cloud / Enterprise tier — explain below

## Checklist

- [ ] `cargo fmt` clean
- [ ] `cargo clippy --no-deps -- -D warnings` clean
- [ ] Tests added or updated, or "no test needed because ___"
- [ ] Documentation updated (README, code comments, or both)
- [ ] No secrets / personal paths committed
- [ ] Conventional Commit message
