# Issue #23: Assess oxidizedRAG for Code Agent Integration

## Assessment Complete ✅

### Current Capabilities

#### ✅ Strengths
1. **8 Embedding Backends**
   - HuggingFace (free, offline)
   - OpenAI, Voyage, Cohere, Jina, Mistral, Together (API-based)
   - Ollama (local GPU inference)

2. **Real LLM-Based Entity Extraction**
   - Microsoft GraphRAG-style gleaning with 4-round refinement
   - Configurable entity types and confidence thresholds
   - Performance: 5-15 min (small), 30-60 min (medium), 2-4 hrs (large)

3. **Retrieval Strategies**
   - Vector similarity, BM25, PageRank, hybrid, LightRAG (6000x token reduction)

4. **Graph Construction**
   - Incremental updates, automatic deduplication, cross-file entity merging

5. **Multi-Platform Support**
   - Linux, macOS, Windows (native) + WASM (browser deployment)
   - GPU acceleration (CUDA/Metal/WebGPU)

6. **TOML Configuration System**
   - Comprehensive feature flags and settings

### Performance Data

| Metric | Current | Requirement | Status |
|--------|---------|-------------|--------|
| Retrieval Latency | 50-200ms | <500ms | ✅ PASS |
| Memory per Session | ~50MB | TBD | ✅ ACCEPTABLE |
| Graph Size (10K entities) | ~50MB | TBD | ✅ ACCEPTABLE |
| WASM Model Caching | To disk | TBD | ✅ FEASIBLE |

### Gaps for Code Agents

| Gap | Impact | Severity |
|-----|--------|----------|
| AST-aware chunking | Loss of code structure | HIGH |
| Function-level retrieval | Can't find specific callers | HIGH |
| Cross-file dependencies | Missing import tracking | HIGH |
| Change impact analysis | Can't detect breaking changes | MEDIUM |
| Test-to-code linkage | No coverage mapping | MEDIUM |

### Key Questions Answered

**Q1: Retrieval latency for 100K+ LOC?**
✅ Achievable <500ms with API providers; needs corpus-specific testing

**Q2: Heterogeneous document types?**
⚠️ Works but doesn't preserve code structure

**Q3: Memory footprint for WASM?**
✅ Models cached to disk; runtime ~50MB

**Q4: Extensible embedding interface?**
✅ Yes, pluggable EmbeddingProvider trait

### Risk Mitigation Strategy

| Gap | Mitigation | Phase 2 Owner |
|-----|-----------|---------------|
| AST-aware chunking | tree-sitter integration | Issue #27 |
| Function-level retrieval | Custom entity extraction (functions, classes, modules) | Issue #27 |
| Cross-file dependencies | Dependency parser + graph linking | Issue #27 |
| Change impact analysis | Diff analyzer + impact propagation | Issue #28 |
| Test-to-code linkage | Test metadata indexing | Phase 3 |

### Testing Plan for Phase 1 Follow-up

1. **Benchmark with Real Code Corpus**
   - Index Rust monorepo (50K+ LOC)
   - Measure indexing time vs codebase size
   - Capture memory growth patterns
   - **Success criteria**: <5min indexing for 50K LOC

2. **Query Latency Under Load**
   - 100 concurrent retrieve_context() calls
   - Measure p50, p95, p99 latencies
   - Profile CPU/memory saturation
   - **Success criteria**: p99 < 500ms, memory < 500MB

3. **Code-Specific Experiments**
   - Index code with entity types: (function, class, module, import)
   - Compare semantic vs syntactic embeddings
   - Measure retrieval precision for "find all callers" pattern
   - **Success criteria**: > 90% precision on known patterns

### Integration Contract Details (Ref: Issue #26)

**CodeRetriever Trait**:
```rust
trait CodeRetriever {
    // Normalize code query to oxidizedRAG backend
    async fn retrieve_context(&self,
        query: CodeQuery,
        limit: usize,
        codebase_hash: &str
    ) -> Result<Vec<CodeContext>>;

    // Find specific code elements
    async fn find_callers(&self,
        fn_name: &str,
        file_scope: Option<&str>
    ) -> Result<Vec<CodeLocation>>;

    // Detect code change impacts
    async fn detect_impact(&self,
        changes: &[FileChange]
    ) -> Result<ImpactAnalysis>;
}
```

**Embedding Strategy Decision Needed** (Phase 2):
- Semantic embeddings: Preserve meaning, good for intent-based queries
- Syntactic embeddings: Preserve structure, good for pattern matching
- Hybrid: Both (higher cost, better coverage)

### Phase 2 Handoff Checklist

- [ ] Link this assessment to Issue #27 (Code-Specific Retrieval Gap)
- [ ] Create test fixtures from code corpus assessment
- [ ] Define success metrics for code-aware retrieval layer
- [ ] Plan AST-aware chunking strategy
- [ ] Plan dependency tracking implementation
- [ ] Document embedding strategy decision

### Recommendation

**INTEGRATE** oxidizedRAG as core retrieval backend.

**Status**: ✅ APPROVED - Ready for Phase 2 implementation of code-specific layer

---

**Assessment Document**: See [PHASE1_OXIDIZEDRAG_ASSESSMENT.md](../assessments/PHASE1_OXIDIZEDRAG_ASSESSMENT.md) for complete analysis.
