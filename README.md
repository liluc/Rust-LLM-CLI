# LLM-Powered CLI

**Student names:**  
Ruitong Li, 1006912815  
Yingxuan Hu, 1006881377

**Contact email:**  
ruiton.li@mail.utoronto.ca  
alvin.hu@mail.utoronto.ca

---

# Video Slide Presentation

[Link to be added: Video presentation explaining the project architecture, features, and demonstration]

---

# Video Demo

[Link to be added: Screencast demonstration of the LLM CLI in use]

---

# Final Report

## Table of Contents

- [1. Motivation](#1-motivation)
- [2. Objectives](#2-objectives)
- [3. Features](#3-features)
- [4. User Guide](#4-user-guide)
- [5. Reproducibility Guide](#5-reproducibility-guide)
- [6. Contributions by Each Team Member](#6-contributions-by-each-team-member)
- [7. Lessons Learned and Concluding Remarks](#7-lessons-learned-and-concluding-remarks)

---

## 1. Motivation

A lack of a lightweight, LLM-powered CLI exists in the current ecosystem. Existing solutions like Codex CLI and AIChat are often too heavy or tied to other ecosystems, leaving a gap for a simple alternative. This gap matters because many developers who choose Rust do so for its speed, safety, and efficiency in building small, reliable tools. When they want to experiment with AI-driven workflows, they often have no choice but to rely on bulky tools.

This project addresses that gap while remaining enjoyable to build. It brings together three areas we want to practice deeply: systems programming in Rust, text-based user interface design, and integration of large language models into developer tools. The project is both rewarding to implement and relevant to ongoing conversations about how developers can interact with AI in their day-to-day work.

By setting a realistic scope, we created a polished prototype that demonstrates novelty without being overwhelming. The novelty lies in its Rust-first design, its ability to demonstrate agentic workflows in a lightweight way, and its practical integration into terminal-based development workflows.

---

## 2. Objectives

The main objective is to build a lightweight, Rust-based CLI powered by local LLM inference. The CLI supports context-aware sessions, integrates with developer tools, and showcases agentic workflows that feel practical and safe.

**Core objectives achieved:**

1. **Stateful CLI Context**: The CLI maintains system-level context (working directory, repository path, command history) and semantic context (recent command outputs). When you run "show status" and then type "commit it", the system correctly links "it" to the shown diff through reference detection and context injection.

2. **Local Inference with Ollama**: All LLM inference runs locally via Ollama, avoiding remote APIs. Streaming output makes responses feel responsive. Health checks at startup ensure Ollama is running and required models are available.

3. **Full-Screen TUI with Ratatui**: The interface provides a clean conversation log, fixed input area, and status bar showing model, working directory, and runtime state. Text wraps properly, and the UI updates smoothly during streaming.

4. **Tiered Intent Resolution**: A 3-tier system resolves user intent from natural language: Tier 1 uses fuzzy matching (< 1ms), Tier 2 uses keyword+embedding hybrid (~50ms), and Tier 3 uses a small LLM classifier (~500ms). The system defaults to conversational chat when uncertain, providing natural fallback.

5. **Agentic Workflows with Safety**: Multi-step workflows like "save work" execute a plan (stage, commit, push) only after showing a preview and receiving confirmation. Users can accept or override suggested commit messages. This keeps automation transparent and intentional.

6. **Repository Awareness**: The CLI automatically detects project types (Rust, Node.js, Python, Go) and git repositories, tailoring commands accordingly. It can show status/diffs, generate commit messages, and search for TODOs.

---

## 3. Features

### Terminal UI and Conversation Layout

Built with Ratatui, the full-screen interface displays messages labeled as System, User, or Assistant. Long lines wrap properly. The input area stays fixed at the bottom, and a status bar shows the active model, working directory, and runtime state. Users can scroll with PgUp/PgDn and navigate input history with Up/Down arrows.

### Chat Model and Streaming Output

Uses Ollama for local inference with streaming output—text appears progressively rather than all at once. Default model is `llama3`, configurable via environment variable or config file. Startup health checks verify Ollama connectivity and model availability.

### Session Context and Semantic References

Tracks working directory, repository root, and project type. Maintains input history and a window of recent outputs (last 5, max 2000 chars each). When reference words like "it", "that", or "the diff" are detected, relevant context is injected into the LLM prompt. This enables natural follow-up: "show status" followed by "commit it" works as expected.

### Tiered Intent Resolution

**Tier 1 (< 1ms)**: Fuzzy matching against learned aliases, exact tool names, and examples. Resolves 90% of common commands instantly.

**Tier 2 (~50ms)**: Hybrid scoring combining keyword matching (60% weight) and semantic embeddings (40% weight). Requires `nomic-embed-text` model. Handles paraphrasing and synonyms.

**Tier 3 (~500ms)**: Small LLM classifier (default: `qwen2:1.5b`) handles novel phrasing. Falls back to chat when uncertain.

Learned commands are stored in `.llm-cli/learned.toml` and matched instantly in future sessions.

### Command Routing and Safety

Supports two modes toggled with Ctrl+S:
- **Chat Mode**: Natural language processed through intent system or sent to LLM
- **Shell Mode**: Direct shell command execution

In Chat mode, prefix commands with `$` or `!` for shell execution. Bang shortcuts (`!!` for last command, `!prefix` for last command starting with prefix) provide bash-style convenience. When the LLM suggests shell commands, users must confirm before execution.

### Git Workflows

**Save work**: Shows plan with status preview, asks confirmation, stages changes, generates commit message based on staged diff, lets user accept/override message, commits, and pushes. Execution report shows results.

**Status**: Shows `git status --short` and `diff --stat` summary. User can type a file path to view that file's diff.

**Commit**: Commits without push, with LLM-generated message suggestion.

**Stage**: Stages changes with confirmation.

**Draft commit message**: Generates commit message from staged changes.

### File Operations and Code Search

**Show file**: Displays file contents in conversation (e.g., "show src/main.rs")

**List files**: Lists directory contents (e.g., "list files in src")

**Write file**: Creates/overwrites files with confirmation (e.g., "write to notes.txt with content: ...")

**Find TODOs**: Ripgrep-based search for TODO/FIXME comments (requires `rg` on PATH)

### Custom Command Learning

When the LLM generates shell commands, users can execute them (`y`), save them as learned commands (`s`), or skip (`n`). Learned commands are matched instantly (< 1ms) in Tier 1 on subsequent uses, enabling personalized workflows.

### Ghost Text Completion

As users type, ghost text suggestions appear based on frecency-ranked history, learned commands, and tool examples. Press Tab to accept. Completions are mode-aware (Chat vs Shell).

### Embedding Cache

Pre-computed embeddings for tool examples are cached in `.llm-cli/embeddings.toml`, providing 10-20x startup speedup (from ~1-2s to ~0.1s). Cache invalidates automatically if the embedding model changes.

---

## 4. User Guide

### Basic Usage

After starting the CLI, the full-screen interface opens with a conversation panel at top and input area at bottom.

**Essential keyboard shortcuts:**
- Type naturally and press Enter to chat
- Up/Down: Navigate input history
- Ctrl+S: Toggle between Chat and Shell modes
- Tab: Accept ghost text completion
- PgUp/PgDn: Scroll conversation
- Esc/q/Ctrl+C: Quit

**Git workflows:**
```
status                    # Show git status and diffstat
save work                 # Stage, commit with generated message, and push
commit                    # Commit without push
stage all                 # Stage all changes
draft commit message      # Generate commit message from staged changes
```

**File operations:**
```
show src/main.rs         # Display file contents
list files in src        # List directory
write to notes.txt with content: ...  # Create/overwrite file
```

**Code search:**
```
find todos               # Search TODO/FIXME (requires ripgrep)
```

**Shell mode:**
- Toggle with Ctrl+S to run shell commands directly
- Built-in: `cd <path>`, `pwd`

**Bang shortcuts in Chat mode:**
```
$ ls -la                 # Execute shell command
! pwd                    # Execute shell command (alternative)
!!                       # Repeat last shell command
!cargo                   # Run last command starting with "cargo"
```

**Custom commands:**

When the LLM suggests shell commands, type:
- `y` or `yes`: Execute
- `s` or `save`: Save as learned command
- `n` or `no`: Skip

**Configuration:**

Create `.llm-cli/config.toml` in your project directory:

```toml
model = "llama3"
embedding_model = "nomic-embed-text"
classifier_model = "qwen2:1.5b"

llm_timeout_secs = 45
cmd_timeout_secs = 60

streaming = true
generate_commit_message = true
```

Or use environment variables:
```bash
export LLM_CLI_MODEL="llama3:8b"
export LLM_CLI_STREAMING="true"
```

All project-specific files are stored in `.llm-cli/` directory (history, learned commands, embeddings, frecency data).

---

## 5. Reproducibility Guide

The instructor will follow these steps on Ubuntu Linux server and macOS Sonoma. The project is a Rust terminal application built with Cargo. Chat features depend on Ollama running locally. Git features require running inside a git repository. TODO search requires ripgrep.

### Prerequisites (Both Platforms)

- Rust toolchain (via rustup)
- Ollama 0.13+ installed and running
- At least one model pulled (we use `llama3`)
- Git (for repository features)
- Ripgrep (optional, for TODO search)

### macOS Sonoma Setup

**1. Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Restart terminal after installation
```

**2. Verify installation:**
```bash
rustc --version
cargo --version
```

**3. Install Ollama:**
```bash
brew install --cask ollama
# Launch Ollama from Spotlight to start the service
```

**4. Pull models:**
```bash
ollama pull llama3
ollama pull nomic-embed-text   # Optional but recommended for semantic matching
```

**5. Install ripgrep (optional):**
```bash
brew install ripgrep
```

**6. Build and run:**
```bash
cd /path/to/project
cargo build
cargo run
```

### Ubuntu Linux Server Setup

**1. Install dependencies:**
```bash
sudo apt update
sudo apt install -y build-essential curl git
```

**2. Install Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
```

**3. Verify installation:**
```bash
rustc --version
cargo --version
```

**4. Install Ollama:**
```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**5. Start Ollama (in separate terminal session):**
```bash
ollama serve
```

**6. Pull models (in second terminal):**
```bash
ollama pull llama3
ollama pull nomic-embed-text
```

**7. Install ripgrep (optional):**
```bash
sudo apt install -y ripgrep
```

**8. Build and run:**
```bash
cd /path/to/project
cargo build
cargo run
```

### Optional Configuration

**Use a different model:**
```bash
export LLM_CLI_MODEL=llama3.2:3b
ollama pull llama3.2:3b
cargo run
```

**Custom config:**

Create `.llm-cli/config.toml` in your project directory (see User Guide for format).

### Quick Sanity Checks

After the UI opens:

1. **Test chat**: Type "Give me a two sentence description of Rust" and confirm streaming response appears
2. **Test history**: Press Up to recall last prompt, edit, and resend
3. **Test git** (if in repo): Type `status` to show git status and diffstat
4. **Test file operations**: Type `show Cargo.toml` or `list files in src`
5. **Test TODO search** (if ripgrep installed): Type `find todos`

### Common Issues and Solutions

**"Ollama daemon unreachable"**
- Run `ollama serve` in a separate terminal

**"Model not found"**
- Run `ollama pull llama3`

**"find todos" not working**
- Install ripgrep: `brew install ripgrep` (macOS) or `apt install ripgrep` (Ubuntu)

**Git features not working**
- Ensure you're inside a git repository and git is installed

**"embeddings: disabled" in status bar**
- Run `ollama pull nomic-embed-text` to enable semantic matching
- Or ignore—basic features work without embeddings

---

## 6. Contributions by Each Team Member

### Yingxuan Hu

**Ollama Integration and Streaming:**
- Implemented Ollama client with streaming output and health checks
- Built server-sent events parsing for real-time responses
- Added error categorization (unreachable vs model missing)

**Intent Resolution System:**
- Designed 3-tier intent resolution architecture
- Built Tier 2 keyword+embedding hybrid classifier
- Implemented Tier 3 LLM-based classifier
- Created embedding cache system for 10-20x startup speedup
- Documented system in `docs/INTENT_SYSTEM.md` and `docs/EMBEDDING_CACHE.md`

**Repository Features:**
- Implemented git workflows (status, save work, commit, stage, draft message)
- Built commit message generation from staged diffs
- Added TODO/FIXME scanning via ripgrep
- Implemented repository and project type detection

**Semantic Context Resolution:**
- Designed Cursor-style context injection system
- Implemented reference word detection and recent outputs tracking
- Enabled natural follow-up interactions ("commit it" after "show status")
- Documented in `docs/SEMANTIC_CONTEXT.md`

**Custom Command Learning:**
- Built command extraction from LLM responses
- Implemented learned aliases with TOML persistence
- Integrated custom commands into Tier 1 for instant recall

**Testing and Documentation:**
- Added unit tests for key modules
- Wrote comprehensive documentation
- Created health check script and reproducibility guide

### Ruitong Li

**Terminal UI System:**
- Designed and implemented full-screen Ratatui interface
- Built three-panel layout (conversation, input, status bar)
- Implemented text wrapping, scroll management, and ghost text rendering
- Created `TerminalGuard` RAII pattern for reliable cleanup

**Application Core:**
- Architected main event loop and state coordination
- Implemented async channel-based background task system
- Built keyboard and mouse event handling
- Created workflow state machine for multi-step confirmations
- Added graceful shutdown handling

**Input Processing:**
- Implemented mode-aware input history with Up/Down navigation
- Built bang shortcuts (`!!`, `!prefix`) with bash-style expansion
- Added built-in commands (`cd`, `pwd`) with path expansion
- Created input deduplication

**Completion System:**
- Designed ghost text completion provider
- Built frecency-based ranking for suggestions
- Integrated with history, learned commands, and tool examples
- Added Tab key acceptance

**Configuration and Session:**
- Built hierarchical configuration system (defaults, TOML file, env vars)
- Implemented session state tracking (cwd, repo root, messages, outputs)
- Added project type detection integration
- Created message streaming state management

**Developer Tools:**
- Implemented shell command execution with subprocess management
- Built file operations (read, write, list) with safety boundaries
- Created tools registry for centralized definitions
- Added proper error handling and user-facing messages

**Mode Switching and Polish:**
- Implemented Chat/Shell mode toggle with visual indicators
- Added shell command shortcuts (`$`, `!`) in Chat mode
- Fixed terminal state issues, race conditions, and scroll bugs
- Improved error messages throughout codebase

---

## 7. Lessons Learned and Concluding Remarks

**Streaming output matters for perceived performance.** Even when a model takes the same time to finish, showing text as it arrives makes the tool feel faster and more responsive. Implementing streaming pushed us to design clearer separation between rendering, state, and background work.

**Agentic behavior must be trustworthy.** Developers are comfortable with automation when they can predict outcomes and execution is intentional. Our plan-and-execute approach with confirmations demonstrates that multi-step workflows can be powerful while keeping users in control.

**Repository awareness doesn't need complexity.** Simply detecting the repo root, exposing status/diff previews, and generating commit messages from staged diffs already supports common tasks effectively. This incremental approach delivers value early and provides a clear path for future enhancements.

**Tiered intent resolution balances speed and flexibility.** By combining fuzzy matching (< 1ms), semantic embeddings (~50ms), and LLM classification (~500ms) with natural fallback to chat, we created a system that feels fast for common commands while gracefully handling novel phrasing. Most users develop habits, so optimizing for the common case while providing flexibility for exploration creates a system that adapts to user behavior over time.

**Terminal UI stability requires attention to detail.** Handling raw mode correctly, keeping layout consistent, wrapping text, truncating large outputs, and showing clear error messages all matter for reliability. These details became especially important because the tool must remain readable while actively streaming output.

**Startup performance has outsized impact.** The embedding cache system's 10-20x speedup transformed the tool from "slow to start" to "instant," fundamentally changing how willing we were to restart it frequently. Performance optimizations that improve iteration speed matter significantly for developer tools.

**Simple heuristics can enable natural interaction.** Semantic context resolution via reference detection doesn't require sophisticated NLP. Simple string matching for pronouns and explicit references combined with a small queue of recent outputs created a surprisingly effective system for follow-up interactions.

**Clear boundaries between LLM and deterministic code matter.** The model is strongest at explanation and summarization, while deterministic commands are best for actions like reading files, searching code, and running git operations. Keeping those roles separate made the tool easier to reason about and safer to use.

**Future directions** include deeper repository analysis (AST parsing, symbol indexing), multi-file editing with diff previews, integration with external tools via MCP (Model Context Protocol), collaborative features (sharing learned commands), and performance optimization for large repositories.

The project achieved its goal of creating a lightweight, Rust-native LLM-powered CLI that feels natural in developer workflows. More importantly, it demonstrated that local inference, semantic intent matching, and careful UX design can combine to create tools that augment rather than replace human decision-making.
