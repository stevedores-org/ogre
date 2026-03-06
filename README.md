# 🧟 OGRE - Orchestrated GraphRAG Engineering

**Integration wiring layer for autonomous code agents within the [lornu.ai](https://github.com/lornu-ai/lornu.ai) ecosystem.**

OGRE connects knowledge retrieval (oxidizedRAG), workflow orchestration (oxidizedgraph), and persistent storage (data-fabric) to enable agents that understand, analyze, and safely modify code.

---

## 🎯 Mission

Enable autonomous code agents to be **productive, safe, and explainable** by architecting the integration between:
- **oxidizedRAG**: Knowledge graph retrieval at scale
- **oxidizedgraph**: Agent workflow orchestration
- **data-fabric**: Persistent knowledge & audit trails
- **lornu.ai**: Multi-cloud autonomous infrastructure

Within the broader lornu.ai ecosystem for infrastructure automation.

## 📋 What's Here

- **[PLAN.md](./PLAN.md)** - Integration wiring plan with 5 phases:
  - Phase 1: Assessment of oxidizedRAG, oxidizedgraph, data-fabric
  - Phase 2: Gap analysis for code-specific agent workflows
  - Phase 3: OGRE architecture & integration contracts
  - Phase 4: Prototype (autonomous PR reviewer)
  - Phase 5: Production hardening & deployment

## 🔑 Key Questions

1. **Retrieval**: How can we retrieve relevant code context at scale?
2. **Orchestration**: How do we define complex agent workflows?
3. **Execution**: How do agents safely analyze and modify real code?
4. **Learning**: How do agents improve through iteration?
5. **Observability**: How do we debug agentic decision-making?

## 🚀 Quick Start

```bash
# Read the evaluation plan
cat PLAN.md

# Start with Phase 1: Assessment
# Survey oxidizedRAG & oxidizedgraph capabilities
```

## 🏗️ Ecosystem Architecture

```
lornu.ai (Multi-cloud autonomous infrastructure)
    ├─ OGRE (Code agent wiring) ← YOU ARE HERE
    │   ├─ oxidizedRAG (retrieval)
    │   ├─ oxidizedgraph (orchestration)
    │   └─ data-fabric (persistence)
    ├─ aivcs (Agent version control)
    ├─ bond (Agent testing)
    └─ lornu-ai-cleaner (Data safety)
```

## 📚 Core Projects

**Integration Targets**:
- [oxidizedRAG](https://github.com/stevedores-org/oxidizedRAG) - Knowledge graph retrieval (code-aware)
- [oxidizedgraph](https://github.com/stevedores-org/oxidizedgraph) - Workflow orchestration (agent lifecycle)
- [data-fabric](https://github.com/stevedores-org/data-fabric) - Schema governance & persistence

**Supporting Infrastructure**:
- [aivcs](https://github.com/stevedores-org/aivcs) - Agent run tracking & version control
- [bond](https://github.com/stevedores-org/bond) - Agent testing framework
- [lornu.ai](https://github.com/lornu-ai/lornu.ai) - Multi-cloud platform & shared agent swarm

## 💡 What OGRE Does

OGRE is **not** a framework — it's the **wiring** that enables:

1. **Context Retrieval**: Query codebases semantically via oxidizedRAG
2. **Workflow Execution**: Run agent decisions through oxidizedgraph
3. **Safe Modifications**: Execute code changes with validation & rollback
4. **Knowledge Persistence**: Store agent learnings in data-fabric
5. **Auditable Decisions**: Full traceability of agent reasoning
6. **Integration**: Connect to lornu.ai's autonomous agent swarm

## 🤝 Contributing

This is an active integration project. Contributions welcome in:
- Gap analysis & assessment
- Integration contract design
- Prototype implementation
- Testing & validation
