# Macro/Custom Command Feature

## Overview

When the intent system can't figure out what you want to do, you can define your own **shell command macros** on the fly. These are saved and reused automatically.

---

## How It Works

### When Nothing Matches

If all 4 tiers fail to understand your input, you'll see:

```
I'm not sure what you want to do with: "deploy staging"

Did you mean:
  [1] save_work - Stage all changes, generate a commit message, commit, and push to remote
  [2] status - Show git status and diff summary
  [3] commit - Commit staged changes with a message (without pushing)
  [4] stage - Stage files for commit using git add
  [5] find_todos - Search for TODO and FIXME comments in the codebase
  [6] run_tests - Run the project's test suite using cargo test
  [7] show_file - Display the contents of a file
  [8] draft_commit_message - Generate a commit message based on staged changes

Options:
  • Type a number to select a tool
  • Type 'macro: <commands>' to define a shell command sequence
    Example: macro: git add -A && git commit -m 'update' && git push
  • Type 'none' to skip

I'll remember your choice for next time.
```

### Define a Macro

You have **two ways** to define macros:

**Option 1: Natural Language (LLM-assisted)**

Just describe what you want in plain English:

```
> stage and commit only, no push

🤔 Generating command for: "stage and commit only, no push"...

💡 Generated command:
  git add -A && git diff --cached --stat && git commit

This will be saved as: "stage and commit only, no push" → shell command

Options:
• Type 'yes' to confirm and execute
• Type 'edit: <new command>' to modify
• Type 'no' to cancel

> yes

✓ Learned macro: "stage and commit only, no push" → git add -A && git diff --cached --stat && git commit
Executing now...
```

**Option 2: Explicit Command**

Type `macro:` followed by your exact shell command(s):

```
> macro: ssh staging 'cd /app && git pull && systemctl restart app'

✓ Learned macro: "deploy staging" → ssh staging 'cd /app && git pull && systemctl restart app'
Executing now...
```

### Next Time

The macro is saved and executes instantly:

```
> deploy staging
[Tier 1, < 1ms] → Executing macro: ssh staging 'cd /app && git pull && systemctl restart app'

> stage and commit only
[Tier 1, < 1ms] → Executing macro: git add -A && git diff --cached --stat && git commit
```

---

## Examples

### Example 1: Deployment Workflow

**Natural language approach:**
```
> deploy to production

I'm not sure what you want to do with: "deploy to production"
...

> deploy to production server

🤔 Generating command for: "deploy to production server"...

💡 Generated command:
  git push origin main && ssh prod 'cd /app && ./deploy.sh'

> yes

✓ Learned macro: "deploy to production" → git push origin main && ssh prod 'cd /app && ./deploy.sh'
Executing now...
```

**Or explicit approach:**
```
> deploy to production

I'm not sure what you want to do with: "deploy to production"
...

> macro: git push origin main && ssh prod 'cd /app && ./deploy.sh'

✓ Learned macro: "deploy to production" → git push origin main && ssh prod 'cd /app && ./deploy.sh'
Executing now...
```

**Every time after:**
```
> deploy to production
→ Executes instantly (< 1ms)
```

### Example 2: Complex Build & Test

**First time:**
```
> run full check

I'm not sure what you want to do with: "run full check"
...

> macro: cargo fmt && cargo clippy && cargo test && cargo build --release

✓ Learned macro: "run full check" → cargo fmt && cargo clippy && cargo test && cargo build --release
Executing now...
```

**Every time after:**
```
> run full check
→ Executes your custom pipeline
```

### Example 3: Database Operations

```
> backup database

I'm not sure what you want to do with: "backup database"
...

> macro: pg_dump mydb > backup_$(date +%Y%m%d).sql && echo "Backup complete"

✓ Learned macro: "backup database" → pg_dump mydb > backup_$(date +%Y%m%d).sql && echo "Backup complete"
```

### Example 4: Docker Workflow

```
> restart containers

I'm not sure what you want to do with: "restart containers"
...

> macro: docker-compose down && docker-compose up -d && docker-compose logs -f

✓ Learned macro: "restart containers" → docker-compose down && docker-compose up -d && docker-compose logs -f
```

---

## Macro Syntax

Macros support **any valid shell command syntax**:

### Multiple Commands (Sequential)
```
macro: command1 && command2 && command3
```

### Multiple Commands (Parallel)
```
macro: command1 & command2 & command3
```

### Conditionals
```
macro: test -f config.toml && echo "Config exists" || echo "No config"
```

### Pipes
```
macro: git log --oneline | head -10
```

### Subshells
```
macro: echo "Today is $(date)" && whoami
```

### Environment Variables
```
macro: export NODE_ENV=production && npm run build
```

---

## Where Macros Are Stored

Macros are stored **separately** from tool aliases for better organization:

### Global Files
```
~/.config/llm-cli/
  ├── learned.toml    # Tool mappings (e.g., "yeet" → save_work)
  └── macros.toml     # Custom shell commands
```

### Per-Project Files
```
/path/to/your/project/.llm_cli/
  ├── learned.toml    # Project-specific tool mappings
  └── macros.toml     # Project-specific shell commands
```

### File Formats

**learned.toml** (tool mappings):
```toml
[[aliases]]
phrase = "yeet"
tool = "save_work"
timestamp = "1702053600"
source = "user_feedback"
```

**macros.toml** (shell commands):
```toml
[[macros]]
phrase = "deploy staging"
command = "ssh staging 'cd /app && git pull && systemctl restart app'"
timestamp = "1702053600"
source = "user_macro_generated"

[[macros]]
phrase = "run full check"
command = "cargo fmt && cargo clippy && cargo test && cargo build --release"
timestamp = "1702053700"
source = "user_macro"
```

---

## Macro vs. Shell Prefix

| Feature | Macro (`macro: ...`) | Shell Prefix (`$ ...`) |
|---------|---------------------|----------------------|
| **Usage** | Learn once, reuse with natural language | Direct execution every time |
| **Syntax** | `macro: <commands>` | `$ <commands>` |
| **Saved?** | Yes, permanently | No |
| **Speed (after learning)** | < 1ms | < 1ms |
| **Use Case** | Frequently used complex workflows | One-off commands |

### When to Use Macros
- ✅ Complex multi-command sequences
- ✅ Frequently repeated operations
- ✅ Want natural language trigger
- ✅ Team can share via `.llm_cli/learned.toml`

### When to Use Shell Prefix
- ✅ One-off commands
- ✅ Quick system checks
- ✅ Don't want to save
- ✅ Already know exact command

---

## Advanced Use Cases

### 1. Project-Specific Shortcuts

Create `.llm_cli/learned.toml` in your project:

```toml
[[aliases]]
phrase = "deploy"
tool = "shell"
source = "user_macro"
macro_command = "./deploy.sh --env production"

[[aliases]]
phrase = "setup dev"
tool = "shell"
source = "user_macro"
macro_command = "cp .env.example .env && docker-compose up -d && npm install"
```

Now "deploy" means different things in different projects!

### 2. Interactive Macros

Macros can include interactive commands:

```
> macro: vim src/main.rs
```

The editor opens in the same terminal.

### 3. Conditional Workflows

```
> macro: cargo test && cargo build --release || echo "Tests failed!"
```

Only builds if tests pass.

### 4. Multi-Step with Pauses

```
> macro: echo "Step 1: Building..." && cargo build && sleep 2 && echo "Step 2: Testing..." && cargo test
```

---

## Managing Macros

### View All Macros

```bash
# Global macros
cat ~/.config/llm-cli/macros.toml

# Global tool aliases
cat ~/.config/llm-cli/learned.toml
```

Or for project-specific:

```bash
# Project macros
cat .llm_cli/macros.toml

# Project tool aliases
cat .llm_cli/learned.toml
```

### Edit Macros

Manually edit the TOML files:

```bash
# Edit macros
vim ~/.config/llm-cli/macros.toml

# Edit tool aliases
vim ~/.config/llm-cli/learned.toml
```

### Delete a Macro

Remove the `[[macros]]` block from `macros.toml`.

### Share Macros

Check the `.llm_cli/` directory into git:

```bash
git add .llm_cli/
git commit -m "Add team aliases and macros"
git push
```

Team members will automatically use the same tool aliases and macros!

**Files to share:**
- `.llm_cli/learned.toml` - Tool mappings (e.g., "status" → status tool)
- `.llm_cli/macros.toml` - Custom shell commands (e.g., "deploy" → deployment script)

---

## Tips

1. **Start simple** - Define basic macros first, make them more complex over time

2. **Use descriptive names** - "deploy staging" is better than "ds"

3. **Test macros first** - Use `$ ...` to test your command before making it a macro

4. **Project-specific macros** - Put deployment/build macros in `.llm_cli/learned.toml`

5. **Global macros** - Put system utilities in `~/.config/llm-cli/learned.toml`

6. **Quote carefully** - Use proper shell quoting for complex commands:
   ```
   macro: ssh server 'cd /app && ./script.sh'
   ```

7. **Combine with tools** - You can still use built-in tools alongside macros

---

## Comparison to Other Tools

### vs. Shell Aliases

**Shell aliases** (`.bashrc`, `.zshrc`):
```bash
alias deploy='git push && ssh prod ./deploy.sh'
```

**LLM CLI macros**:
```
macro: git push && ssh prod ./deploy.sh
```

| Feature | Shell Aliases | LLM CLI Macros |
|---------|--------------|----------------|
| Natural language | No | Yes |
| Per-project | Manual setup | Automatic |
| Share via git | Manual export | Just commit `.llm_cli/learned.toml` |
| Quick to define | Edit file | Define in-app |
| Typo tolerance | No | Yes (fuzzy match) |

### vs. Makefiles

**Makefile**:
```makefile
deploy:
    git push && ssh prod ./deploy.sh
```

```bash
make deploy
```

**LLM CLI**:
```
> deploy staging
```

| Feature | Makefiles | LLM CLI Macros |
|---------|-----------|----------------|
| Natural language | No | Yes |
| Setup required | Yes (`Makefile`) | No |
| Language-agnostic | Yes | Yes |
| Learning curve | Higher | Lower |

---

## Security Considerations

⚠️ **Macros execute shell commands directly**

- Review macros before executing
- Be careful with commands that:
  - Delete files (`rm -rf`)
  - Modify system files
  - Access sensitive data
  - Run as root (`sudo`)

- Project macros (`.llm_cli/learned.toml`) are code:
  - Review in pull requests
  - Don't commit sensitive data (passwords, keys)

---

## Troubleshooting

### Macro not executing

**Check the saved command:**
```bash
cat ~/.config/llm-cli/learned.toml | grep -A 5 "your phrase"
```

**Verify shell syntax:**
```
$ your command here
```

If it works with `$`, it should work as a macro.

### Wrong macro executing

**Check for duplicates:**
```bash
grep "phrase = \"your phrase\"" ~/.config/llm-cli/learned.toml
grep "phrase = \"your phrase\"" .llm_cli/learned.toml
```

Project-local macros override global ones.

### Want to update a macro

1. Delete the old entry from TOML
2. Re-define with `macro: new command`

Or edit the TOML file directly.

---

## Summary

Macros let you:
- ✅ Define custom shell workflows on the fly
- ✅ Use natural language to trigger them
- ✅ Share them with your team via git
- ✅ Have different macros per project
- ✅ Execute instantly after first definition (< 1ms)

Combined with the 4-tier intent system, you have:
1. **Built-in tools** (save_work, status, etc.)
2. **Learned tool mappings** (natural language → tools)
3. **Custom macros** (natural language → shell commands)

The system becomes **your personal assistant**, understanding what you mean and executing complex workflows with simple phrases.

