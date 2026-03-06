# OGRE CI/CD Setup

**Local validation + GitHub Actions reporting**

---

## Local Development Setup

### Install local-ci

```bash
# Install local-ci globally
curl -fsSL https://github.com/stevedores-org/local-ci/releases/download/v0.3.0/local-ci-linux-amd64 \
  -o ~/.local/bin/local-ci && chmod +x ~/.local/bin/local-ci

# Verify installation
local-ci --version
```

### Configure Git Hooks

```bash
# Configure git to use .githooks directory
git config core.hooksPath .githooks

# Pre-commit hook is now active
# (validates files before each commit)
```

---

## Running Checks Locally

### Default Checks (before commit)

```bash
# Runs automatically via pre-commit hook
git commit -m "your message"

# Or manually:
local-ci --profile pre-commit
```

### Full CI Profile

```bash
# What GitHub Actions runs
local-ci --profile ci

# Or just run specific stage:
local-ci docs
local-ci links
local-ci format
```

### Auto-Fix Issues

```bash
# Fix formatting (trailing whitespace, etc)
local-ci --fix

# Then re-run:
local-ci --profile ci
```

### Dry-Run Mode

```bash
# See what would run without executing
local-ci --dry-run

# List all available stages
local-ci --list
```

---

## GitHub Actions Workflow

### Triggered On

- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

### What It Does

1. **Validates Documentation**
   - Checks PLAN.md, README.md, TEAM_SUMMARY.md exist
   - Validates markdown formatting
   - Checks internal links

2. **Comments on PRs**
   - Posts validation results
   - Links to local-ci documentation

### Files

- `.github/workflows/local-ci.yml` - Main workflow definition
- Automatically posts summary comment on PRs

---

## Phase Roadmap

### Current (Phase 1-2)

✅ Documentation validation
- ✅ Pre-commit hook
- ✅ GitHub Actions workflow
- ✅ PR comments

### Future (Phase 3+)

When Rust workspace is added:

```bash
# .local-ci.toml will be updated with:
- cargo fmt (format check)
- cargo check (type checking)
- cargo clippy (linting)
- cargo test (testing)
```

Then activate in pre-commit hook:

```bash
# Uncomment in .githooks/pre-commit:
if test -f Cargo.toml; then
    local-ci --profile pre-commit
fi
```

---

## Troubleshooting

### Pre-commit hook fails

```bash
# Disable temporarily (not recommended)
git commit --no-verify

# Or fix issues and retry
local-ci --fix
git add .
git commit -m "fix: address pre-commit issues"
```

### local-ci not found

```bash
# Install to PATH
curl -fsSL https://github.com/stevedores-org/local-ci/releases/download/v0.3.0/local-ci-linux-amd64 \
  -o ~/.local/bin/local-ci && chmod +x ~/.local/bin/local-ci

# Or use absolute path
/path/to/local-ci --version
```

### Wrong hook directory

```bash
# Reconfigure
git config core.hooksPath .githooks

# Verify
git config --get core.hooksPath
```

---

## Key Files

| File | Purpose |
|------|---------|
| `.local-ci.toml` | CI configuration (stages, profiles) |
| `.githooks/pre-commit` | Pre-commit hook script |
| `.github/workflows/local-ci.yml` | GitHub Actions workflow |

---

**Updated**: 2026-03-06
**Status**: ✅ Operational
**Next**: Phase 3 - Update for Rust workspace checks
