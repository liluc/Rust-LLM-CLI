# Autocompletion

Ghost-text completion with smart path detection, similar to Cursor or GitHub Copilot.

## Core Structure

```
src/completion.rs     CompletionProvider - fuzzy matching + path completion
src/app.rs           Ghost text state + Tab key handling  
src/ui.rs            Ghost text rendering (dim gray after cursor)
```

## Completion Sources

1. **Tool examples** - 50+ phrases from `TOOLS[]` (e.g., "push my changes", "show status")
2. **Command history** - Recent commands (last 50)
3. **Learned aliases** - User-taught commands from `.llm-cli/learned.toml`
4. **File/folder paths** - Smart path completion when typing paths

## Smart Detection

The system decides completion mode based on input:

### Full Input Matching
When typing commands, it matches the entire input:
- `"sav"` → ghost: `"e work"` (fuzzy matches "save work")
- `"push my ch"` → ghost: `"anges"` (completes to "push my changes")

### Path Completion
When the last word looks like a path, it completes files/folders:
- `"show src"` → ghost: `"/"` (directory exists)
- `"read src/ma"` → ghost: `"in.rs"` (file completion)
- `"ls ./"` → ghost: `"src/"` (relative path)

A word is path-like if it:
- Contains `/` or `\`
- Starts with `.` or `~`
- Matches an existing file/folder in cwd

## Usage

- **Tab** - Accept ghost text
- **Type normally** - Ghost text updates in real-time
- **Backspace** - Ghost text updates as you edit

## Implementation

`CompletionProvider::get_ghost_completion()` returns only the **suffix** to append. Fuzzy matching uses `SkimMatcherV2`, path completion uses `std::fs::read_dir()` for fast directory listing.

