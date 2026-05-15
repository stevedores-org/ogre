# Phase 2 Gap: Code-Specific Retrieval

**Status**: Reviewed 🟢
**Assignee**: claude-code
**Issue**: #27
**Branch**: feature/phase2-gap-code-retrieval
**Phase 1 Source**: `assessments/PHASE1_OXIDIZEDRAG_ASSESSMENT.md`
**Related**: PLAN.md Phase 2.1

---

## Executive Summary

oxidizedRAG embeds code as text. That's enough for "find a chunk that looks like this query" but not for "find every caller of `fn_name` and tell me which tests cover them." This document specifies how OGRE closes that gap **on top of** oxidizedRAG, without forking it: a thin `CodeRetriever` trait, an AST-aware indexing pipeline that fans tree-sitter output into typed entities, a dependency graph stored alongside the existing entity graph, and an impact-analysis query that joins both.

Five capabilities, five action items, one foundational decision (fq-name canonicalization), and three open questions tracked separately. None of the design here requires changes inside oxidizedRAG core; it consumes oxidizedRAG as a backend and adds a code-semantics layer above it.

---

## Phase 1 Recap (what we are building on)

Phase 1 confirmed:

- tree-sitter is already a feature flag in `graphrag-core` (`code-chunking = ["tree-sitter", "tree-sitter-rust"]`) but **the output is not threaded into retrieval ranking** — chunks are embedded generically.
- Entity extraction is LLM-based and works on text, with no notion of "this entity is a function declared at line N."
- Incremental indexing exists at the document level, not at the function level.
- Storage backends include SurrealDB; sufficient for adding a typed dependency graph alongside the existing entity graph.

Phase 2 gap (#27) is therefore the *integration* gap, not a missing-feature gap inside oxidizedRAG.

---

## Foundational Decision: FqName canonicalization

The `fq_name` is load-bearing: it is the primary key for every entity, the endpoint of every edge, and the input to every retrieval method. Phase 3 schema work cannot begin without a settled form, so this is a **decision**, not an open question.

**Canonical form**: `<lang>:<scheme>` where `<scheme>` is the language-native fully-qualified path.

| Language | Scheme | Example |
|---|---|---|
| Rust | `crate::module::item` (`::`-separated) | `rust:lornu_ai::agent::run` |
| Rust trait impl | `<Type as Trait>::method` | `rust:<MyService as Handler>::handle` |
| TypeScript | `package/module#export` | `ts:@lornu/agent/run#default` |
| TypeScript class method | `package/module#Class.method` | `ts:@lornu/agent/run#Agent.start` |
| Python | `package.module.item` (`.`-separated) | `py:lornu.agent.run` |
| Python class method | `package.module.Class.method` | `py:lornu.agent.Agent.start` |

**Rules**:

1. **Treated as opaque strings** outside the parser that produced them. Joins on `fq_name` are exact-equality only — no cross-language prefix matching.
2. **Anonymous / closure entities** are addressed by structural path: `rust:lornu_ai::agent::run::{closure#0}`, `ts:@lornu/agent/run#default.{anon@line:42}`. Stable across reindex if the surrounding structure is stable; unstable otherwise (accepted limitation).
3. **Generics** are recorded at their declaration site only (`rust:lornu_ai::cache::Cache::<K, V>::get`). Monomorphized variants are not materialized as distinct entities; resolution against them surfaces via `Resolution::Heuristic` (see trait types below).
4. **Macro-generated items**: addressed by the expanding macro's name plus a structural index (`rust:lornu_ai::derive_handler!::Generated#0`). Treated as `Resolution::Heuristic` until a macro-expansion pass is added (out of scope for #27).

**Versioning**: the canonical form is `fq_name_v1`. Any change is a breaking schema change and requires a reindex. The version is stored as a global property on the index, not per-entity.

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

**Storage rationale**: SurrealDB in `data-fabric` is chosen because (a) oxidizedRAG's existing text-embedding entity graph is already there, so structural joins (`Function → embedded_chunk`, `Test → covers → Function`) stay in-process with no cross-store consistency problem; (b) SurrealQL handles recursive graph traversal natively, which the call-graph closure in `impact()` needs; (c) no second store to operate. The tradeoff is performance: a dedicated graph DB (Neo4j, RocksDB-backed custom) would be faster on deep traversals. If `impact()` bench targets (see Performance) are not met, swap the call-graph store to a typed key-value backend behind the same trait. This is an implementation detail of `ogre-codegraph`, not a design commitment.

**Resolution policy**: name resolution is intentionally conservative. Each edge carries a `Resolution` categorical (see "Type sketches" below): `Resolved` (unique fq-name match), `Heuristic { reason, candidates }` (e.g., trait method with N candidate impls; macro expansion approximate match), or `Unresolved { receiver_text }` (dyn dispatch with no inferable type, opaque function pointer). Confidence is the categorical itself — there is no scalar score, because static analysis does not produce one in a principled way. `Heuristic` carries a human-readable `reason` and the candidate `fq_name`s the resolver considered. Phase 4 / 5 work can refine the heuristic resolvers; this phase only requires that we never silently drop an edge or pretend `Heuristic` is `Resolved`.

### 4. Change impact analysis

**Goal**: given a diff, return the set of *potentially affected* functions and tests.

**Approach** (two-pass, bounded):

1. **Direct hit**: parse the diff, map each changed line to an `fq_name` via line-range index; emit the changed functions.
2. **Transitive close**: from the direct hits, traverse `calls`-edges in reverse up to `N` hops. Two bounds are applied during traversal:
   - **Node cap**: stop once `result_cap` (default 500) distinct nodes have been collected; mark the result as `truncated: true`.
   - **High-fan-in skip**: a node whose in-degree exceeds `fanin_skip` (default 100) on the `calls` edge is recorded in the result but its callers are *not* expanded. Rationale: in a real codebase the high-in-degree node is usually a utility (`unwrap`-equivalent, a logger, an error constructor) and following it explodes the closure without adding signal. The skipped expansion is reported in `ImpactSet.skipped_fan_in`.
3. **Test linkage**: join the affected set against `references` from test functions (capability 5) to surface failing tests. Tests are added to the result regardless of whether the node cap was hit (tests are always small in count).

`N` is configurable. Default `N=2` (direct callers + their callers); past that, signal-to-noise on monorepos collapses fast. Both caps are also tunable per-call so PR Reviewer (Phase 4) can ask for a wider sweep on small diffs.

### 5. Test-to-code linkage

**Goal**: given `fq_name`, return tests that exercise it; given a failing test, return code suspects.

**Detection strategy** (in order of preference):

1. **Attribute-based** (preferred when the language has a test attribute):
   - Rust: `#[test]` attribute on a function; `#[cfg(test)] mod tests { ... }` block — every function inside the test-cfg module is a test.
   - Python: `pytest` collects by `test_*` prefix + the import resolves to `pytest`; treat as attribute-equivalent.
   - TypeScript: `describe()` / `it()` / `test()` callbacks invoked at module top-level — detected by call-graph, not by name.
2. **Name-based fallback** (only for languages without a test attribute system):
   - Go: `_test.go` filename + `Test*` exported function.
   - Generic: `test_*` prefix or `*_test.{ts,py}` filename — flagged with `Resolution::Heuristic` since it relies on convention.

A function flagged `is_test: true` records its detection source (`attribute` / `cfg_module` / `name_pattern` / `framework_callback`) so downstream consumers can filter out heuristic matches when high precision is needed.

The `calls` graph already gives us the connection between a test and the functions it exercises (within the limits of static call resolution).

**Coverage data is out of scope for #27.** When a coverage file (`lcov`, `coverage.json`) is available, it can be ingested as additional edges with `source: "coverage"` — covered explicitly in a future ticket, not here.

---

## CodeRetriever Trait (proposed)

```rust
pub trait CodeRetriever {
    /// Resolve a textual symbol query to a canonical fully-qualified name.
    fn resolve(&self, query: &SymbolQuery) -> Result<Vec<FqName>, CodeRetrieverError>;

    /// Fetch the definition entity for a function or type.
    fn definition(&self, fq: &FqName) -> Result<Option<CodeEntity>, CodeRetrieverError>;

    /// Direct callers (1-hop) on the call graph.
    fn callers(&self, fq: &FqName) -> Result<Vec<CallerEdge>, CodeRetrieverError>;

    /// Transitive impact closure given a set of changed entities. Bounded.
    fn impact(&self, changed: &[FqName], opts: &ImpactOptions) -> Result<ImpactSet, CodeRetrieverError>;

    /// Tests that statically exercise the given fq names.
    fn covering_tests(&self, fq: &[FqName]) -> Result<Vec<FqName>, CodeRetrieverError>;

    /// Fuzzy / semantic retrieval — delegates to oxidizedRAG.
    fn semantic_search(&self, q: &str, k: usize) -> Result<Vec<CodeEntity>, CodeRetrieverError>;
}
```

The first five methods are structural; the sixth is the existing oxidizedRAG retrieval, untouched. This is the minimum surface area Phase 4 (PR Reviewer prototype) requires to "find context, then reason."

### Type sketches

```rust
/// Opaque canonical name. Constructed only by the language-specific parser.
/// String form follows §"Foundational Decision: FqName canonicalization".
pub struct FqName(String);

/// Lookup intent.
pub struct SymbolQuery {
    pub text: String,
    pub kind: Option<EntityKind>,   // Function | Type | Module | Test | Any
    pub lang: Option<Language>,     // None = search across all languages
    pub mode: QueryMode,            // Exact | Suffix | Prefix
}

/// What a static resolver was able to determine for a given edge.
pub enum Resolution {
    Resolved,
    Heuristic { reason: String, candidates: Vec<FqName> },
    Unresolved { receiver_text: String },
}

pub struct CallerEdge {
    pub from: FqName,
    pub call_site: LineRange,
    pub resolution: Resolution,
}

pub struct ImpactOptions {
    pub hops: usize,            // default 2
    pub result_cap: usize,      // default 500
    pub fanin_skip: usize,      // default 100
    pub include_tests: bool,    // default true
}

pub struct ImpactSet {
    pub direct: Vec<FqName>,
    pub transitive: Vec<FqName>,
    pub tests: Vec<FqName>,
    pub skipped_fan_in: Vec<FqName>,   // nodes hit but not expanded
    pub truncated: bool,                // true if result_cap was reached
}

pub enum CodeRetrieverError {
    SymbolNotFound(String),
    AmbiguousSymbol { query: String, candidates: Vec<FqName> },
    IndexBehind { latest_commit: Option<String> },   // staleness signal
    UnsupportedLanguage(String),
    Storage(StorageError),
}
```

These are sketches, not the final API — `ogre-codegraph` (#33) finalizes them. But the shapes settle three things the design needs to commit to: (a) `truncated` and `skipped_fan_in` are first-class outputs of `impact()`, not implicit; (b) `Resolution` is categorical, not a scalar score; (c) `IndexBehind` is a distinct error variant so PR Reviewer can reason about whether to wait or proceed with stale data.

---

## Action Items (mapped to #27 checklist)

| # | #27 action item | Concrete deliverable in this phase |
|---|---|---|
| 1 | Design code-aware indexing strategy | This document, sections "AST-aware chunking" + "Dependency tracking" |
| 2 | Extend oxidizedRAG with AST support | **No core fork**. New `ogre-code-index` crate (Phase 3) that wraps oxidizedRAG, drives tree-sitter, writes structural metadata into the existing entity graph |
| 3 | Build dependency graph layer | New `ogre-codegraph` crate (Phase 3); SurrealDB schema in `data-fabric` for `Function`, `Module`, `Type`, `Calls`, `Imports`, `ReferencesType` |
| 4 | Add impact analysis module | Implemented in `CodeRetriever::impact`; backed by a SurrealQL traversal over the call graph, bounded per §4 |
| 5 | Performance testing on 100K+ LOC | Bench harness in a follow-up ticket; budget targets in "Performance" section below |

Phase 2 (this issue) ships the **design spec** — the action items above are the input for the Phase 3 design issues (#33, #32) and the Phase 4 prototype (#37).

---

## Open Questions (remaining)

Q1 (cross-language fq-name) is settled — see "Foundational Decision" above. The remaining questions stay tracked in #43:

1. **Macro / generic / dyn-dispatch resolution depth.** Default: record as `Resolution::Heuristic` or `Resolution::Unresolved` per §3; do not stall ingestion. Phase 4 may add deeper heuristics (e.g. trait-impl resolution for monomorphized traits where the candidate set is small). Open: when do we promote a heuristic to resolved? Suggested rule: monomorphized trait with exactly one impl in the crate graph → `Resolved`; everything else stays `Heuristic`.

2. **Reindex granularity on change.** Default: per-function. A single-line edit reindexes one chunk + recomputes the local subgraph. Acceptable upper-bound work for a change is the impact set at `hops=N`. Open: what is the file-level reindex threshold (e.g., > 10 changed functions in one file → reindex whole file)?

3. **Embedding model for code chunks vs prose.** Default: reuse oxidizedRAG's configured model unchanged. Code-specific embedding models (CodeBERT, StarCoder-embed) are a Phase 5 hardening question, not a gap-analysis one.

---

## Performance budget

Targets are stated per scale point. Methodology: derived from Phase 1's < 500 ms agent-workflow budget, working backward — an agent typically issues `resolve` + `impact` + `semantic_search` per decision, and the round-trip must leave headroom for the LLM call (~300 ms minimum). The 100K-LOC column is the primary target; 10K and 1M are sanity bounds.

| Operation | 10K LOC p99 | 100K LOC p50 | 100K LOC p99 | 1M LOC p99 (stretch) |
|---|---|---|---|---|
| `resolve(symbol)` | < 5 ms | < 5 ms | < 50 ms | < 200 ms |
| `callers(fq, 1-hop)` | < 20 ms | < 20 ms | < 200 ms | < 1 s |
| `impact(N=2)` on diff of < 50 lines | < 50 ms | < 200 ms | < 1 s | < 5 s |
| Full reindex | < 30 s | < 5 min | < 15 min | < 2 h |
| Incremental reindex (single file) | < 1 s | < 2 s | < 10 s | < 30 s |

**Cache assumptions**: `resolve` and `callers` targets are warm-index (in-memory name index loaded; this costs roughly 100 MB at 100K LOC, 1 GB at 1M LOC). Cold-start adds 2–10 s before any query is fast. `impact()` traversal is not cached.

**Bench harness target**: `lornu-ai/lornu.ai` (current Rust+TS codebase, ~80K LOC) is the candidate for the 100K-LOC column. A synthetic 1M-LOC fixture (e.g. duplicated graphrag with renamed crates) is sufficient for the 1M column.

These are targets, not commitments. If `impact()` cannot hit < 1 s at 100K LOC with SurrealDB, the storage rationale in §3 commits to swapping the call-graph backend; the budget is what holds us accountable.

---

## Failure model

How the indexer behaves under bad inputs and partial state:

- **Tree-sitter parse failure on a file**: file is skipped, logged with `language`, `path`, `error`; an entity of kind `UnparseableFile` is written so the file is visible in queries but carries no structural edges. Reindex on next pass.
- **Ingestion lag / index behind HEAD**: indexer exposes an `index_watermark` (commit sha + timestamp). `CodeRetriever` operations check the watermark and return `CodeRetrieverError::IndexBehind` if the requested ref is newer. Callers (Phase 4 PR Reviewer) decide whether to wait or proceed with stale data — the decision is *not* in the retriever's hands.
- **Ambiguous symbol resolution**: `resolve` returns `Vec<FqName>` rather than `FqName` precisely to surface this. `AmbiguousSymbol` error variant is for the case where the *intent* of the query implied a unique name but multiple candidates exist (e.g., `definition()` on an overloaded name).
- **Partial graph during write**: writes go through a single transaction per file ingestion; readers see file-level atomicity. A file with 50 functions either appears with all 50 or with none.
- **Stale edges after rename**: rename is treated as delete + insert, not as a single edge update. Edges from the old `fq_name` are deleted in the same transaction that inserts the new entity. Cross-file references that have not yet been reingested may briefly point to a non-existent target; these are surfaced as `Resolution::Unresolved` until the referring file is reingested.

---

## What This Does Not Cover

- **Runtime call graphs** (profile-guided): static resolution only.
- **Coverage ingestion**: see capability 5; deferred.
- **Code-specific embedding model selection**: see open question 3; Phase 5.
- **Multi-repo retrieval**: single-repo first; cross-repo joins are a separate gap.
- **Real-time on-keystroke indexing**: incremental on commit / on save is the upper bound.
- **Macro expansion pass**: macro-generated entities use heuristic addressing per §"Foundational Decision"; full expansion-aware indexing is a separate ticket.

These are intentional non-goals for #27 and should each become their own issue if pursued.

---

## Dependencies on Other Issues

- **#32** (Phase 3 OGRE Core - Agent Lifecycle Engine): consumes `CodeRetriever` from the planner.
- **#33** (Phase 3 OGRE Retrieval - Code-Aware Integration Layer): is the implementation of the trait specified here.
- **#28** (Phase 2 Gap: Safe Action Execution): impact-analysis output feeds the safety layer's "what tests must pass before this change ships."
- **#43** (Meta: Open Questions): remaining open questions 1–3 above are cross-posted there for tracking.

---

## Conclusion

The retrieval gap is a layering gap, not a missing-engine gap. oxidizedRAG provides text-embedding retrieval; the `CodeRetriever` trait above provides structural retrieval; impact analysis is the join.

Implementation order recommended for Phase 3:
1. Schema in `data-fabric` (Function / Module / Type entities; Calls / Imports / ReferencesType edges) — keyed by the canonical `fq_name` form from §"Foundational Decision".
2. `ogre-code-index` crate — tree-sitter ingestion → entity writes.
3. `ogre-codegraph` crate — SurrealQL traversals, `CodeRetriever` impl with the bounded `impact()` from §4.
4. Bench harness against a real 100K-LOC repo (`lornu-ai/lornu.ai` is the candidate).

**Status of #27**: design complete; remaining open questions tracked in #43. Prototype work tracked in #33 / #37.

---

**Created**: 2026-05-14
**Revised**: 2026-05-15 — fq-name canonicalization promoted to a decision; trait type sketches added; `impact()` bounded; resolution made categorical; storage rationale added; failure model added; test detection split into attribute vs name fallback.
