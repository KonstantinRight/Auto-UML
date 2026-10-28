# Repository guidance

## Project overview

Auto-UML is a Rust library and CLI that parses source code with tree-sitter,
extracts a shared diagram model, resolves types across files, and renders Mermaid
or Graphviz class diagrams. Use a recent stable Rust toolchain supporting edition
2024 (Rust 1.85 or newer).

Read `CONTRIBUTING.md` for contribution conventions. Read
`docs/navigating-the-parser.md` before changing extraction logic and
`docs/add-a-language.md` before adding language support.

## Code map

- `src/main.rs`: CLI arguments, language detection, grammar selection, input and
  output orchestration, and remote repository handling.
- `src/lib.rs`: public module exports and library-facing documentation.
- `src/diagram.rs`: shared classes, functions, and variables; CST traversal and
  extraction into the diagram model.
- `src/lang_config.rs`: `LangConfig` schema and embedded configuration loading.
- `src/stitcher.rs`: directory traversal, diagram merging, and cross-file type
  resolution.
- `src/mermaid.rs` and `src/graphviz.rs`: output renderers and renderer unit tests.
- `languages/<lang>/config.yaml`: language-specific tree-sitter node patterns and
  extraction options.
- `languages/<lang>/test.yaml` and `examples/`: integration-test mappings, source
  fixtures, and expected Mermaid output in `.uml` files.
- `languages/stitch_test/`: multi-file resolution fixtures.
- `tests/integration_tests.rs`: language test harness and stitcher integration test.
- `build.rs`: stages language configurations into Cargo's `OUT_DIR` for embedding
  with `include_dir!`. Edit source YAML files, never the staged copies.

## Implementation conventions

- Prefer language configuration changes when existing `LangConfig` options can
  express the behavior. Keep shared parsing logic applicable across languages.
- Treat `LangConfig` as the authoritative YAML schema. When adding an option,
  preserve behavior for configurations that omit it and document its meaning.
- Keep extraction, cross-file resolution, and rendering in their respective
  modules. Consider both renderers when changing the shared diagram model.
- Preserve ignore-aware directory traversal and explicit language selection when
  changing discovery or CLI behavior.
- Add regression coverage for parser, resolution, or rendering behavior changes.
  Review expected `.uml` changes for correctness; do not update fixtures merely
  to make a failing test pass. Fixture comparisons trim surrounding whitespace
  but otherwise compare the generated Mermaid text exactly.
- Keep CLI usage and supported-language documentation in `README.md` and
  `src/lib.rs` consistent with implementation changes.
- Follow existing Rust style and rustfmt. Use Conventional Commits with a scope
  where useful, such as `fix(cpp): preserve generic parameters`.

## Adding a language

Follow `docs/add-a-language.md` in full. The main touchpoints are:

1. Add a compatible tree-sitter grammar dependency in `Cargo.toml` and update
   `Cargo.lock` through Cargo.
2. Define source fixtures and expected `.uml` files for `simple_struct`,
   `impl_block`, `complex_types`, and `generics`, mapped by `test.yaml`.
3. Add `config.yaml`, including file extensions and extraction patterns.
4. Register grammar selection in `src/main.rs` and in the integration harness's
   `setup_parser`; invoke `lang_tests!` for the new fixture directory.
5. Verify configuration names, CLI aliases, and test names agree. In particular,
   `setup_parser` matches the exact `name` from `test.yaml`.
6. Update supported-language documentation and run the new language's tests plus
   the full suite to catch shared-parser regressions.

## Build and validation

Run commands from the repository root: integration fixture paths are relative
to the working directory.

```text
cargo build --release
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-targets --all-features
cargo test --doc
```

For focused iteration, use `cargo test rust`, `cargo test rust::generics`, or
`cargo test test_stitcher_integration`. `cargo test --no-fail-fast` runs unit,
integration, and documentation tests without stopping at the first failed target.
Run the relevant full checks before delivering code changes; report any checks
that could not run. Documentation-only edits do not require a Rust build.

`prek.toml` defines formatting, spelling, YAML/TOML validation, and Cargo hooks.
If setting up hooks, use prek 0.4.10 or newer and `prek install`; see
`CONTRIBUTING.md` for hook stages and details.

For a CLI smoke test, use an explicit destination under the ignored build folder:

```text
cargo run -- --lang rust --source-code src/ --destination target/agent-smoke.mmd
cargo run -- --lang rust --source-code src/ --format graphviz --destination target/agent-smoke.dot
```

## Working tree and generated files

- Check `git status --short` before editing and preserve unrelated local changes.
- Keep build output under `target/`; do not commit build artifacts or temporary
  diagrams.
- `UML.md` is a generated repository diagram, refreshed by a GitHub workflow on
  relevant changes to `main`. Regenerate it only when the task calls for it, and
  preserve any existing local edits. Avoid default CLI destinations during tests
  because they can overwrite diagrams in the repository root.
- Keep changes focused on the requested task; do not include unrelated fixture
  rewrites, dependency upgrades, or formatting churn.
