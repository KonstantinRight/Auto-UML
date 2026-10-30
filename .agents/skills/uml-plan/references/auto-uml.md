# Auto-UML CLI reference

Upstream: https://github.com/EthanDGee/Auto-UML

This integration uses the local CLI, not an HTTP API or assumed MCP tool. Multi-input commands were checked against this repository's implementation. Check the invoked executable's help; an installed release may predate this interface.

## Discover and run

```text
auto-uml --help
auto-uml --version
auto-uml --list-languages
auto-uml --source-code "path/to/module" --format mermaid --destination "path/to/artifacts/current.mmd"
auto-uml --lang rust --source-code "path/to/models" "path/to/service.rs" --destination "path/to/artifacts/current.mmd"
auto-uml --lang rust -s "path/to/models" -s "path/to/service.rs" --destination "path/to/artifacts/current.mmd"
```

`--source-code` / `-s` accepts one or more space-separated files or directories, or repeated flags. Quote paths containing spaces; commas are literal path characters. Each invocation produces one diagram, deduplicating repeated files and overlapping directories and resolving types across the selected files. Without local inputs or `--git`, the source defaults to `.`; local inputs and `--git` are mutually exclusive.
The checked build supports Rust, Java, JavaScript, TypeScript, C++, C#, Objective-C, Dart, and Kotlin; discover support with `--list-languages`.

Use the smallest meaningful scope retaining relevant cross-file context. For separate directories, run separately or analyze their common ancestor and label a filtered presentation. Inspect the output after success: an empty or partial diagram does not prove that relationships are absent. Discover support rather than assuming all languages work.

## Output and provenance

- `--format mermaid` produces raw source to embed inside a Mermaid fence.
- `--code-block` wraps output in a Markdown fence; do not double-wrap it.
- `--format graphviz` is available when DOT output is useful.
- Always pass `--destination`; defaults can create UML.mmd, UML.md, or UML.dot in the working directory.
- Directory analysis respects ignore files. Explicit files are included even when ignored. Use `--no-ignore` only when ignored directory contents are intentionally needed.
- `--git <URL>` clones and analyzes a remote repository. Prefer local source for local planning to include working changes.

Keep raw artifacts separate from simplified or corrected views. For syntax problems, preserve the original, fix only the presentation copy, and disclose the adaptation. Check renderability with an available Mermaid renderer when practical; otherwise state rendering was not checked.

Multi-input labels use paths relative to the selected roots' common ancestor solely for naming, or drive-qualified paths across Windows drives. Path punctuation is escaped in class identifiers to preserve distinct identities; source-file display labels retain the original paths. Preserve generated identifiers in the raw baseline rather than simplifying them into potentially colliding names.

## Missing tool or failures

Check PATH and known local installations before declaring the tool missing. Upstream installation uses `cargo install auto-uml`; installation and network access remain subject to permissions and the user's scope. Do not silently install global dependencies merely to avoid explaining their absence.

Inspect failures and make a focused correction for a bad path, language selection, or destination problem. Avoid repeated installs or broad configuration changes. If unavailable or still failing, report the cause, continue useful inspection, and clearly label any source-derived fallback. Record parser limitations instead of inventing missing relationships.

Multi-input analysis fails before writing output for invalid paths, traversal or read errors, or an empty eligible selection. A pre-existing destination may therefore be stale after failure; verify exit status before presenting it as the new baseline.
