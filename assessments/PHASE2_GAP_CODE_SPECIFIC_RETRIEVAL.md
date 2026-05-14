# Phase 2 Gap: Code-Specific Retrieval

**Status**: Draft 🟡
**Assignee**: claude-code
**Issue**: #27
**Branch**: feature/phase2-gap-code-retrieval
**Phase 1 Source**: `assessments/PHASE1_OXIDIZEDRAG_ASSESSMENT.md`
**Related**: PLAN.md Phase 2.1

---

## Executive Summary

oxidizedRAG embeds code as text. That's enough for "find a chunk that looks like this query" but not for "find every caller of `fn_name` and tell me which tests cover them." This document specifies how OGRE closes that gap **on top of** oxidizedRAG, without forking it: a thin `CodeRetriever` trait, an AST-aware indexing pipeline that fans tree-sitter output into typed entities, a dependency graph stored alongside the existing entity graph, and an impact-analysis query that joins both.

Five capabilities, five action items, four open questions. None of the design here requires changes inside oxidizedRAG core; it consumes oxidizedRAG as a backend and adds a code-semantics layer above it.

---

## Phase 1 Recap (what we are building on)

Phase 1 confirmed:

- tree-sitter is already a feature flag in `graphrag-core` (`code-chunking = ["tree-sitter", "tree-sitter-rust"]`) but **the output is not threaded into retrieval ranking** — chunks are embedded generically.
- Entity extraction is LLM-based and works on text, with no notion of "this entity is a function declared at line N."
- Incremental indexing exists at the document level, not at the function level.
- Storage backends include SurrealDB; sufficient for adding a typed dependency graph alongside the existing entity graph.

Phase 2 gap (#27) is therefore the *integration* gap, not a missing-feature gap inside oxidizedRAG.

---

## Five Capabilities (from #27)

### 1. AST-aware chunking

**Goal**: preserve function / class / block boundaries so a retrieved chunk is always a complete syntactic unit, never a half-function fragment.

**Approach**:
- Use the existing `code-chunking` feature with tree-sitter grammars for the languages we care about (Rust first; TypeScript, Python next).
- Wrap chunks with structural metadata: `{ language, node_kind, fq_name, byte_range, line_range, parent_fq_name }`.
- Store metadata as entity attributes in the oxidizedRAG graph; the chunk text remains the embeddable payload.

**Why this is sufficient**: ranking quality benefits from embeddings on coherent units; structural metadata lets the retriever filter ("only functions") and join (caller-of, called-by) without re-parsing.

### 2. Function / module-level retrieval

**Goal**: queries of shape `"find all uses of fn_name"` and `"return the body of module::path::fn_name"`.

**Approach**:
- Define the `CodeRetriever` trait below; resolve "fn_name" → `fq_name` via a name index built during ingestion.
- For `find_callers(fq_name)`: traverse the **call graph** (capability 3) starting from `fq_name` against the reverse-edge direction.
- For `get_definition(fq_name)`: lookup by `fq_name` in the function index; return entity attributes + chunk text.

Both paths bypass embedding similarity entirely — they're structural lookups. Embeddings remain for fuzzy queries ("functions doing X-like things").

### 3. Dependency tracking

Three graphs, distinct edge kinds, single SurrealDB namespace:

| Edge | From → To | Source |
|---|---|---|
| `imports` | module → module | top-level `use` / `import` statements |
| `calls` | function → function | tree-sitter `call_expression` nodes |
| `references_type` | function → type | parameters, return types, struct fields |

The call graph is the load-bearing edge for impact analysis. The import graph is cheap, fast to compute, and useful for cross-language symbolic queries when full call resolution is infeasible.

**Resolution policy**: name resolution is intentionally conservative. If a call cannot be resolved unambiguously (dyn dispatch, generic indirection, macro-generated code), record it as an **unresolved edge** with the textual receiver and a confidence score; do not silently drop it. Phase 4 / 5 work can refine resolution; gap analysis here only requires that we never pretend we know more than we do.

### 4. Change impact analysis

**Goal**: given a diff, return the set of *potentially affected* functions and tests.

**Approach** (two-pass):
1. **Direct hit**: parse the diff, map each changed line to an `fq_name` via line-range index; emit the changed functions.
2. **Transitive close**: from the direct hits, traverse `calls`-edges in reverse N hops; the closure is the affected set.
3. Join the affected set against `references` from test functions (capability 5) to surface failing tests.

`N` is configurable. Default `N=2` (direct callers + their callers); past that, signal-to-noise on monorepos collapses fast.

### 5. Test-to-code linkage

**Goal**: given `fq_name`, return tests that exercise it; given a failing test, return code suspects.

**Approach**:
- Tag functions whose `fq_name` matches a test convention (`#[test]` attribute, `test_*` prefix, `_test.ts` suffix) as `is_test: true` during ingestion.
- The `calls` graph already gives us the connection between a test and the functions it exercises (within the limits of static call resolution).
- Coverage data is **out of scope for #27**. When a coverage file (`lcov`, `coverage.json`) is available, it can be ingested as additional edges with `source: "coverage"` — covered explicitly in a future ticket, not here.

---

## CodeRetriever Trait (proposed)

```rust
pub trait CodeRetriever {
    /// Resolve a textual symbol query to a canonical fully-qualified name.
    fn resolve(&self, query: &SymbolQuery) -> Result<Vec<FqName>>;

    /// Fetch the definition entity for a function or type.
    fn definition(&self, fq: &FqName) -> Result<Option<CodeEntity>>;

    /// Direct callers (1-hop) on the call graph.
    fn callers(&self, fq: &FqName) -> Result<Vec<CallerEdge>>;

    /// Transitive impact closure given a set of changed entities.
    fn impact(&self, changed: &[FqName], hops: usize) -> Result<ImpactSet>;

    /// Tests that statically exercise the given fq names.
    fn covering_tests(&self, fq: &[FqName]) -> Result<Vec<FqName>>;

    /// Fuzzy / semantic retrieval — delegates to oxidizedRAG.
    fn semantic_search(&self, q: &str, k: usize) -> Result<Vec<CodeEntity>>;
}
```

The first five methods are structural; the sixth is the existing oxidizedRAG retrieval, untouched. This is the minimum surface area Phase 4 (PR Reviewer prototype) requires to "find context, then reason."

---

## Action Items (mapped to #27 checklist)

| # | #27 action item | Concrete deliverable in this phase |
|---|---|---|
| 1 | Design code-aware indexing strategy | This document, sections "AST-aware chunking" + "Dependency tracking" |
| 2 | Extend oxidizedRAG with AST support | **No core fork**. New `ogre-code-index` crate (Phase 3) that wraps oxidizedRAG, drives tree-sitter, writes structural metadata into the existing entity graph |
| 3 | Build dependency graph layer | New `ogre-codegraph` crate (Phase 3); SurrealDB schema in `data-fabric` for `Function`, `Module`, `Type`, `Calls`, `Imports`, `ReferencesType` |
| 4 | Add impact analysis module | Implemented in `CodeRetriever::impact`; backed by a SurrealQL traversal over the call graph |
| 5 | Performance testing on 100K+ LOC | Bench harness in a follow-up ticket; budget targets in "Performance" section below |

Phase 2 (this issue) ships the **design spec** — the action items above are the input for the Phase 3 design issues (#33, #32) and the Phase 4 prototype (#37).

---

## Open Questions (with default answers; flag for reviewer)

1. **Cross-language fq-name scheme.** Rust uses `crate::module::function`; TypeScript uses `package/module#export`; Python uses `package.module.function`. Default: namespace each language's names with a `lang:` prefix (`rust:lornu_ai::agent::run`, `ts:@lornu/agent/run`) and treat them as opaque strings.

2. **Macro / generic / dyn-dispatch.** Default: record as unresolved edges with a confidence score; do not stall ingestion. Phase 4 may add heuristics (e.g. trait-impl resolution for monomorphized traits).

3. **Reindex granularity on change.** Default: per-function. A single-line edit reindexes one chunk + recomputes the local subgraph. Acceptable upper-bound work for a change is the impact set at `hops=N`.

4. **Embedding model for code chunks vs prose.** Default: reuse oxidizedRAG's configured model unchanged. Code-specific embedding models (CodeBERT, StarCoder-embed) are a Phase 5 hardening question, not a gap-analysis one.

---

## Performance budget

| Operation | p50 target | p99 target | Codebase |
|---|---|---|---|
| `resolve(symbol)` | < 5 ms | < 50 ms | 100K LOC |
| `callers(fq, 1-hop)` | < 20 ms | < 200 ms | 100K LOC |
| `impact(N=2)` on diff of < 50 lines | < 200 ms | < 1 s | 100K LOC |
| Full reindex | < 5 min | < 15 min | 100K LOC |
| Incremental reindex (single file) | < 2 s | < 10 s | 100K LOC |

These are targets, not commitments. They derive from the < 500 ms agent-workflow budget identified in Phase 1: an agent will typically call `resolve` + `impact` + `semantic_search` per decision, and each must leave headroom for the LLM round-trip.

---

## What This Does Not Cover

- **Runtime call graphs** (profile-guided): static resolution only.
- **Coverage ingestion**: see capability 5; deferred.
- **Code-specific embedding model selection**: see open question 4; Phase 5.
- **Multi-repo retrieval**: single-repo first; cross-repo joins are a separate gap.
- **Real-time on-keystroke indexing**: incremental on commit / on save is the upper bound.

These are intentional non-goals for #27 and should each become their own issue if pursued.

---

## Dependencies on Other Issues

- **#32** (Phase 3 OGRE Core - Agent Lifecycle Engine): consumes `CodeRetriever` from the planner.
- **#33** (Phase 3 OGRE Retrieval - Code-Aware Integration Layer): is the implementation of the trait specified here.
- **#28** (Phase 2 Gap: Safe Action Execution): impact-analysis output feeds the safety layer's "what tests must pass before this change ships."
- **#43** (Meta: Open Questions): items 1–4 above should be cross-posted there for tracking.

---

## Conclusion

The retrieval gap is a layering gap, not a missing-engine gap. oxidizedRAG provides text-embedding retrieval; the `CodeRetriever` trait above provides structural retrieval; impact analysis is the join.

Implementation order recommended for Phase 3:
1. Schema in `data-fabric` (Function / Module / Type entities; Calls / Imports / ReferencesType edges).
2. `ogre-code-index` crate — tree-sitter ingestion → entity writes.
3. `ogre-codegraph` crate — SurrealQL traversals, `CodeRetriever` impl.
4. Bench harness against a real 100K-LOC repo (lornu.ai itself is a candidate).

**Status of #27**: design complete pending review; prototype work tracked in #33 / #37.

---

**Created**: 2026-05-14
**Draft**: Yes — awaiting review
