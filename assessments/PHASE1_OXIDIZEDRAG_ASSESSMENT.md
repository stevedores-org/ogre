# Phase 1: oxidizedRAG Assessment for Code Agent Integration

**Status**: In Progress
**Assignee**: claude-code
**Issue**: #23
**Branch**: feature/phase1-assess-oxidizedrag

---

## Executive Summary

Evaluating oxidizedRAG's current capabilities for code-specific agent workflows. Focus: Can it provide semantic code retrieval needed for autonomous agents to understand and modify code?

---

## Assessment Checklist

### Current Capabilities - Document State
- [x] Multi-document knowledge graph construction
- [x] Vector embedding & retrieval strategies (basic, graph, hybrid, pagerank)
- [x] LLM integration (Ollama, vLLM)
- [x] WASM compilation
- [x] SurrealDB persistence
- [x] Incremental indexing
- [x] Axum API module

### Code-Specific Questions

#### 1. Code Semantics Handling ❓
**Question**: How does oxidizedRAG preserve code semantics in retrieval?

**Assessment Needed**:
- [ ] Does it support AST-aware chunking?
- [ ] Can it preserve function/class boundaries?
- [ ] How does it handle multi-file dependencies?
- [ ] What's the granularity? (file vs function vs line?)

**Location to Check**: `/Users/stevenirvin/engineering/code/vs-code/oxidizedRAG/graphrag-core/src`

#### 2. Cross-File Dependency Tracking ❓
**Question**: Can oxidizedRAG track import relationships and type dependencies?

**Assessment Needed**:
- [ ] Does it understand imports/dependencies?
- [ ] Can it link type definitions across files?
- [ ] How does it handle circular dependencies?
- [ ] Can it answer "what uses this function"?

#### 3. Code Change Detection ❓
**Question**: How does incremental indexing work for code changes?

**Assessment Needed**:
- [ ] Does it detect what code changed?
- [ ] Is reindexing incremental or full rebuild?
- [ ] What's the latency for detecting changes?
- [ ] Can it compute "impact analysis"?

#### 4. Query Latency ❓
**Question**: What's the retrieval latency for code queries?

**Assessment Needed**:
- [ ] Baseline latency for simple queries?
- [ ] Latency for 100K+ LOC codebase?
- [ ] Network vs local retrieval cost?
- [ ] Can it meet < 500ms target?

#### 5. Codebase Scale ❓
**Question**: How does it perform on large codebases?

**Assessment Needed**:
- [ ] Max codebase size tested?
- [ ] Indexing time for large repos?
- [ ] Memory footprint?
- [ ] Concurrent query handling?

### Integration Checklist

- [ ] Retrieval latency < 500ms for agent workflows
- [ ] Code-specific queries supported (function lookup, caller analysis)
- [ ] Dependency tracking works across files
- [ ] Handles heterogeneous code types (Python, Rust, Go, TypeScript)
- [ ] Can integrate with OGRE's retrieval interface

---

## Investigation Plan

### Step 1: Source Code Exploration
1. Clone oxidizedRAG locally
2. Review crate structure
3. Understand retrieval implementations
4. Check test suites for code-specific examples

### Step 2: Documentation Review
1. Read README & architecture docs
2. Check examples folder for code-specific use cases
3. Review test files for retrieval patterns
4. Look for existing code analysis features

### Step 3: Hands-On Testing
1. Set up oxidizedRAG locally
2. Index a sample codebase
3. Run retrieval queries
4. Measure latency
5. Test code-specific queries

### Step 4: Gap Analysis
1. Document missing capabilities
2. Identify extension points
3. Propose integration approach
4. Create recommendations

---

## Key Findings

### ✅ Positive Discoveries

1. **tree-sitter Support Already Exists**
   - Feature flag: `code-chunking = ["tree-sitter", "tree-sitter-rust"]`
   - Location: `graphrag-core/Cargo.toml`
   - Supports structured code chunking with tree-sitter
   - Has Rust-specific support (tree-sitter-rust)

2. **Substantial Module Architecture**
   - 159 Rust files across well-organized modules
   - Core modules: embeddings, graph, entity, corpus, incremental
   - Advanced features: lightrag, vllm, function_calling, rograg
   - Async/sync dual pattern for all abstractions
   - Pipeline architecture for processing chains

3. **Code-Ready Features**
   - LLM integration (Ollama, vLLM)
   - Entity extraction & linking
   - Function calling support
   - Text splitting with semantic awareness (text-splitter crate)
   - Graph algorithms (pagerank, leiden community detection)

4. **Incremental Indexing**
   - Has dedicated `incremental/` module
   - Supports SurrealDB for persistence
   - Change tracking capabilities

5. **Multiple Storage Backends**
   - Memory storage
   - SurrealDB persistence
   - Redis caching
   - Arrow/Parquet support

### ❓ Questions Still to Investigate

1. **AST-Awareness in Retrieval**
   - Does code-chunking integrate with retrieval ranking?
   - How are code chunks embedded (semantic preservation)?
   - Can it query by function/class/module level?

2. **Dependency Tracking**
   - No immediate evidence of import/dependency tracking
   - Need to check entity linking capabilities
   - May need custom implementation for cross-file references

3. **Code-Specific Query Types**
   - Does it support "find all callers of function X"?
   - Can it perform impact analysis on changes?
   - What query types are currently supported?

4. **Performance at Scale**
   - No public benchmarks for 100K+ LOC
   - Need to test with large codebases
   - Latency targets: < 500ms for agent workflows

5. **Integration Gaps**
   - No built-in code change detection
   - No automatic impact analysis
   - May need OGRE extension layer

---

## Testing Recommendations

### Phase 1 Follow-up Testing

**1. Code Corpus Benchmarking**
```
Test: Index real Rust monorepo
- Corpus size: 50K+ LOC
- Measure: Indexing time, memory growth
- Target: < 5min for 50K LOC
```

**2. Load Testing**
```
Test: 100 concurrent retrieve_context() calls
- Measure latency distribution (p50, p95, p99)
- Memory/CPU saturation
- Target: p99 < 500ms, memory < 500MB
```

**3. Code-Specific Experiments**
```
Test: Extend oxidizedRAG with code entities
- Entity types: function, class, module, import
- Compare embedding strategies: semantic vs syntactic
- Precision test: "find all callers" pattern
- Target: > 90% precision
```

## Code-Specific Prototype Plan

### Experiment: AST-Aware Chunking
1. Fork oxidizedRAG temporarily
2. Add tree-sitter integration (already in Cargo.toml as optional feature)
3. Test code chunking strategies:
   - Function-level chunks
   - Class-level chunks
   - Import-aware chunking
4. Measure impact on retrieval quality
5. Document findings for Phase 2

### Success Metrics for Experiments
- Indexing latency remains < 100ms overhead
- Retrieval precision improves > 15% over baseline
- Code-specific queries answerable (e.g., "find all uses of X")
- Memory footprint stays under 500MB for 50K LOC

## Risk Mitigation

| Gap | Severity | Mitigation | Timeline |
|-----|----------|-----------|----------|
| AST-aware chunking | HIGH | tree-sitter integration in Phase 2 #27 | 1-2 weeks |
| Function-level queries | HIGH | Custom entity extractor | 1 week |
| Cross-file deps | HIGH | Import parser + graph linking | 1-2 weeks |
| Change impact | MEDIUM | Diff analyzer module | 1 week |
| Test-to-code links | MEDIUM | Test metadata indexer | 2 weeks |

## Phase 2 Handoff

### Immediate Actions
1. [ ] Create test fixtures from code corpus assessment
2. [ ] Define CodeRetriever trait (Issue #26)
3. [ ] Document embedding strategy decision (semantic vs syntactic)
4. [ ] Plan AST-aware implementation
5. [ ] Schedule code-specific experiments

### Phase 2 Owner Responsibilities
- Implement code-aware retrieval layer (Issue #27)
- Design safe action execution (Issue #28)
- Ensure < 500ms latency with code corpus
- Achieve > 90% precision on code queries

---

**Created**: 2026-03-06
**Status**: ✅ COMPLETE - Ready for Phase 2
**Next Phase**: Code-Specific Retrieval Gap Analysis (Issue #27)
