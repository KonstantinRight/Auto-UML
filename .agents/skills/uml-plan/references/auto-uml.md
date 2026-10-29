# Auto-UML CLI reference

Upstream: https://github.com/EthanDGee/Auto-UML

This integration uses the local CLI, not an HTTP API or assumed MCP tool. Commands were checked against upstream documentation and the installed CLI on 2026-10-28. Prefer installed help if flags differ.

## Discover and run

```text
auto-uml --help
auto-uml --version
auto-uml --list-languages
auto-uml --source-code "path/to/module" --format mermaid --destination "path/to/artifacts/current.mmd"
```

`--source-code` accepts one file or directory. Detection is automatic; use `--lang kotlin` or another listed language when needed. The checked build lists Rust, Java, JavaScript, TypeScript, C++, C#, Objective-C, Dart, and Kotlin. Discover support rather than assuming all languages work.

Use the smallest meaningful scope retaining relevant cross-file context. For separate directories, run separately or analyze their common ancestor and label a filtered presentation. Inspect the output after success: an empty or partial diagram does not prove that relationships are absent.

## Output and provenance

- `--format mermaid` produces raw source to embed inside a Mermaid fence.
- `--code-block` wraps output in a Markdown fence; do not double-wrap it.
- `--format graphviz` is available when DOT output is useful.
- Always pass `--destination`; defaults can create UML.mmd, UML.md, or UML.dot in the working directory.
- Directory analysis respects ignore files. Use `--no-ignore` only when ignored sources are intentionally needed.
- `--git <URL>` clones and analyzes a remote repository. Prefer local source for local planning to include working changes.

Keep raw artifacts separate from simplified or corrected views. For syntax problems, preserve the original, fix only the presentation copy, and disclose the adaptation. Check renderability with an available Mermaid renderer when practical; otherwise state rendering was not checked.

## Missing tool or failures

Check PATH and known local installations before declaring the tool missing. Upstream installation uses `cargo install auto-uml`; installation and network access remain subject to permissions and the user's scope. Do not silently install global dependencies merely to avoid explaining their absence.

Inspect failures and make a focused correction for a bad path, language selection, or destination problem. Avoid repeated installs or broad configuration changes. If unavailable or still failing, report the cause, continue useful inspection, and clearly label any source-derived fallback. Record parser limitations instead of inventing missing relationships.
