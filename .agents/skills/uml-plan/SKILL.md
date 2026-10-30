---
name: uml-plan
description: Propose implementations or refactors with Auto-UML current-state diagrams, proposed UML class diagrams, and an actionable Markdown plan. Use for architecture-backed planning and before-and-after UML proposals rather than ordinary code edits.
---

# UML Plan

Produce a reviewable implementation proposal grounded in existing code. A planning request ends with the proposal; implement only when the user also requests implementation. Revise the diagrams and accompanying plan together.

## Capture the current state

Before designing the target structure, read [references/auto-uml.md](references/auto-uml.md) and run Auto-UML on the relevant files or directories. Combine relevant selections with multi-input `--source-code`. Write to explicit destinations in a task artifact or temporary directory. Retain the exact generated baseline and record the analyzed paths, command, and available version information.

Extraction is not a guarantee of completeness. Identify unresolved types and omitted relationships; never silently add inferred edges to a diagram labelled raw Auto-UML output. If readability requires filtering or corrections, label the displayed diagram as adapted and link the retained raw artifact. Use separate diagrams for disjoint scopes rather than concatenating Mermaid documents.

If the tool is unavailable, unsupported, or fails after a focused attempt to resolve the cause, explain the limitation and offer a clearly labelled source-derived fallback. Never claim Auto-UML succeeded when it did not. For entirely new functionality, show the existing integration context rather than inventing existing classes.

Ask about material ambiguity while continuing independent inspection; label reasonable assumptions.

## Propose the implementation

Create a second Mermaid `classDiagram` for the intended implementation. Match the baseline scope and preserve names for unchanged types. Show responsibilities, relevant members, and relationships that explain the change. Distinguish inheritance, realization, dependency, association, and composition by their actual meaning; a field alone does not establish composition.

Mark new and modified elements with readable annotations and a legend. List removals explicitly outside the target diagram so they do not appear to remain in the architecture. Map types to existing or proposed paths. Label provisional interfaces and design assumptions. Add other diagram types only when they clarify behavior the class diagram cannot express.

Explain the rationale, relevant tradeoffs, compatibility, and failure handling. Supply concrete implementation steps and verification criteria. Do not claim unimplemented behavior or unrun tests are validated.

## Present and save

Return the complete document as rendered Markdown directly in chat. Never wrap the entire document in a code fence. Use `mermaid` fences for diagrams inside the document. If rendering is unavailable, preserve portable Mermaid source and explain the limitation.

Use this template as an answer:

1. Getting started:
  - Goal, scope, and assumptions.
  - Current structure: diagram, provenance, and observations.
2. Solution
  - Proposed structure: diagram, legend, and rationale.
  - Changes by file or module, including removals.
  - Ordered implementation checklist.
  - Verification and genuinely unresolved decisions.
3. Summary
  - (If applicable) simplified data flow across entities in the code for the solution
  - A short paragraph-wide summary of the proposed solution and tradeoffs of it.


**Write a concise, direct and focused, dry rather than overexplaining (unless asked to) answer. Keep it short but reasonable. Don't bloat the text and don't repeat yourself or mds in the repo.**