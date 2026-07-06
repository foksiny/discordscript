# DiscordScript

**A super easy Discord bot programming language.**

```
cmd ping {
    slash true
    reply "Pong!"
}
```

DiscordScript is a custom scripting language designed specifically for creating Discord bots. It compiles to a native interpreter (Rust) and can transpile to Python (discord.py, nextcord, py-cord, discloud).

---

## Features

- **Simple syntax** — block-based `{}` syntax, no indentation rules
- **Native interpreter** — run `.ds` files directly with zero dependencies
- **Multi-backend transpilation** — generate Python code for discord.py, nextcord, py-cord, or discloud
- **Built-in modules** — HTTP requests, JSON storage, SQLite database, TTL cache
- **Rich command system** — slash commands, prefix commands, context menus, modals, select menus, buttons, embeds, pagination
- **Event system** — `on ready`, `on member_join`, `on member_leave`, `on message`, custom events
- **Scheduling** — cron expressions and interval-based (`every 30s`) task scheduling
- **Middleware pipeline** — global and per-command middleware with request/response hooks
- **Permissions & cooldowns** — granular permission checks and per-user/per-channel cooldowns
- **Threading & parallelism** — `parallel[...]` blocks for concurrent execution
- **Error handling** — `try`/`catch`/`finally` with explicit `throw`
- **Custom types** — user-defined types with fields, methods, and table bindings
- **Functions** — reusable `fn` definitions with parameters and return types
- **Enums** — algebraic data types with named variants and optional fields
- **Database migrations** — `table`/`migration` blocks for schema definition
- **Models** — type-safe data models with field validation and defaults
- **Flow control** — `if`/`else`, `for` loops, `while` loops, `match` expressions, `break`/`continue`
- **String interpolation** — embed expressions in strings with `{var}`
- **Inline tests** — write `test` blocks alongside your code, run with `discordscript test`
- **Source formatting** — `discordscript fmt` auto-formats your code
- **Static analysis** — `discordscript check` catches errors before runtime
- **Hot-reload** — `--watch` flag auto-restarts on file changes

---

## Quick Start

### Install

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/foksiny/discordscript/main/scripts/install.sh | bash
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://raw.githubusercontent.com/foksiny/discordscript/main/scripts/install.ps1 | iex"
```

**Via Cargo (all platforms):**
```bash
cargo install --git https://github.com/foksiny/discordscript
```

### Create a bot

```bash
discordscript init my-bot
cd my-bot
```

### Write your first command

Edit `main.ds`:

```
config {
    prefix "!"
    status "playing" "DiscordScript"
    intents ["guilds", "messages"]
}

cmd ping {
    slash true
    description "Check if the bot is alive"

    reply "Pong!"
}
```

### Check and run

```bash
discordscript check main.ds
discordscript run main.ds
```

---

## Examples

| File | Description |
|---|---|
| `examples/hello.ds` | Simple bot with ping/greet/info commands, embeds, and events |
| `examples/economy.ds` | Economy bot with balance/daily/leaderboard, cooldowns, DB, pagination |
| `examples/moderation.ds` | Moderation bot with kick/ban/warn/clear, middleware, permissions, JSON storage |
| `examples/advanced.ds` | Full-featured demo: 13 commands, types, threads, pagination, try/catch, webhooks, cache, HTTP, schedules, tests |

### hello.ds

```
config {
    prefix "!"
    status "playing" "DiscordScript"
    intents ["guilds", "messages", "members"]
}

cmd ping {
    slash true
    reply "Pong!"
}

cmd greet {
    slash true
    param user {
        type user
        description "User to greet"
    }
    reply "Welcome, {user.mention}!"
}

on ready {
    reply "Bot is online!" channel "#general"
}
```

### economy.ds

```
config {
    prefix "!"
    db {
        type "sqlite"
        path "data/economy.db"
    }
}

cmd balance {
    slash true
    let user_id = interaction.user.id
    let row = db.query("SELECT balance FROM economy WHERE user_id = ?", [user_id])
    if row == null {
        db.execute("INSERT INTO economy (user_id, balance) VALUES (?, 100)", [user_id])
        reply "Welcome! You received 100 coins."
    } else {
        reply "Your balance: {row[0]} coins"
    }
}
```

### moderation.ds

```
config {
    prefix "!"
    global_middleware ["logging"]
}

middleware "logging" {
    log info "Command executed: {interaction.command.name}"
}

middleware "check_admin" {
    if !interaction.member.permissions.admin {
        cancel "Only administrators can use this command."
    }
}

cmd kick {
    slash true
    permission "kick_members"
    middleware ["check_admin"]
    param user { type user }
    param reason { type string required false }
    discord.kick(user, reason)
    reply "Kicked {user.name}."
}
```

See `tests/fixtures/` for 50+ additional test files covering every language feature.

---

## Language Guide

### Syntax Basics

```
// This is a comment
# This is also a comment

// Variables (immutable by default)
let name = "DiscordScript"
let count = 42
let pi = 3.14
let is_active = true
let list = [1, 2, 3]
let obj = { "key": "value" }

// Mutable variable
let mut counter = 0
set counter = counter + 1

// String interpolation
reply "Hello, {user.name}! You have {count} messages."

// Blocks use curly braces
if count > 10 {
    reply "You have more than 10 messages!"
}
```

### Types

| Type | Description | Examples |
|---|---|---|
| `string` | Text | `"hello"`, `'world'` |
| `int` | Integer | `42`, `-5`, `1_000_000` |
| `float` | Floating point | `3.14`, `-0.5` |
| `bool` | Boolean | `true`, `false` |
| `null` | Null value | `null` |
| `list` | Array | `[1, 2, 3]`, `["a", "b"]` |
| `object` | Key-value map | `{"key": "value"}` |
| `user` | Discord user | `interaction.user` |
| `channel` | Discord channel | `channel "#general"` |
| `role` | Discord role | `role "@everyone"` |
| `member` | Guild member | `interaction.member` |
| `attachment` | File attachment | `interaction.attachments[0]` |

### Custom Types

```
type GuildConfig {
    prefix string
    log_channel string
    welcome_message string default "Welcome!"
    fn get_prefix() {
        return prefix
    }
}

cmd config_server {
    let config = GuildConfig {
        prefix: "!",
        log_channel: "123",
        welcome_message: "Hello!"
    }
    reply "Prefix: {config.get_prefix()}"
}
```

### Enums

```
enum Color {
    Red(string)
    Green(string)
    Blue(string)
    Custom(hex string)
}
```

### Flow Control

```
// If/Else
if score >= 100 {
    reply "You win!"
} else if score >= 50 {
    reply "Almost there!"
} else {
    reply "Keep trying!"
}

// For loops
for item in items {
    reply "Item: {item}"
}

// While loops
let mut i = 0
while i < 5 {
    reply "Count: {i}"
    set i = i + 1
}

// Match expressions
match color {
    "red" { reply "Red!" }
    "blue" { reply "Blue!" }
    else { reply "Other!" }
}

// Break and Continue
for item in items {
    if item == "stop" { break }
    if item == "skip" { continue }
    reply "{item}"
}
```

### Functions

```
fn add(a int, b int) -> int {
    return a + b
}

fn greet(name string) {
    reply "Hello, {name}!"
}
```

### Commands

```
// Slash command
cmd ping {
    slash true
    description "Check if the bot is alive"
    reply "Pong!"
}

// Prefix command
cmd greet {
    prefix true
    param name {
        type string
        description "Name to greet"
    }
    reply "Hello, {name}!"
}

// Command with parameters
cmd echo {
    slash true
    param message {
        type string
        description "Message to echo"
        required true
        max_length 100
    }
    param ephemeral {
        type bool
        description "Send as ephemeral"
        required false
    }
    if ephemeral {
        ephemeral reply "{message}"
    } else {
        reply "{message}"
    }
}

// Command with permissions
cmd ban {
    slash true
    permission "ban_members"
    default_permission false
    param user { type user }
    param reason { type string required false }
    // ...
}

// Subcommands
cmd config {
    subcommand set {
        param key { type string }
        param value { type string }
        reply "Set {key} to {value}"
    }
    subcommand get {
        param key { type string }
        reply "Value: {json.get(key)}"
    }
}
```

### Events

```
on ready {
    log info "Bot is online!"
    status "online"
}

on member_join {
    let welcome_channel = channel "#welcome"
    send "Welcome {user.mention} to the server!" channel "{welcome_channel}"
}

on message {
    if message.content == "ping" {
        reply "pong"
    }
}
```

### Embeds & Components

```
cmd info {
    slash true
    embed {
        title "Bot Information"
        description "Information about this bot"
        color 0x00FF00
        field "Version" "1.0.0" inline true
        field "Library" "DiscordScript" inline true
        footer "Powered by DiscordScript"
        timestamp true
    }
    row {
        button "Primary Button" style "primary" custom_id "btn_primary"
        button "Danger" style "danger"
        url_button "GitHub" url "https://github.com/foksiny/discordscript"
    }
}

modal "Feedback" {
    input "Name" short placeholder "Your name" required true
    input "Feedback" paragraph placeholder "Tell us what you think" max_length 1000
}
```

### Middleware

```
middleware "logging" {
    log info "Command: {interaction.command.name}"
    log info "User: {interaction.user.name}"
}

middleware "check_admin" {
    if !interaction.member.permissions.administrator {
        cancel "Only administrators can use this command."
    }
}

middleware "rate_limit" {
    let key = "rate:{interaction.user.id}"
    let count = cache.get(key) ?? 0
    if count >= 5 {
        cancel "Rate limited. Try again later."
    }
    cache.set(key, count + 1, 60)
    next // Continue to next middleware or command
}

cmd admin_only {
    slash true
    middleware ["check_admin", "logging"]
    // ...
}
```

### Schedules

```
schedule "daily_cleanup" {
    cron "0 0 * * *"
    timezone "America/Sao_Paulo"
    // Runs every day at midnight
    db.execute("DELETE FROM temp_data WHERE expires_at < datetime('now')", [])
    log info "Cleaned up expired data"
}

schedule "status_update" {
    every 30s
    // Runs every 30 seconds
    let count = db.query("SELECT COUNT(*) FROM online_users", [])[0]
    status "watching" "{count} users online"
}
```

### Error Handling

```
cmd trycatch {
    slash true
    try {
        let result = risky_operation()
        reply "Success: {result}"
    } catch err {
        reply "Error: {err}"
        log error "Operation failed: {err}"
    } finally {
        log info "Operation attempted"
    }
}

// Explicit throw
if something_wrong {
    throw "Something went wrong!"
}
```

### Database

```
// SQLite database
config {
    db {
        type "sqlite"
        path "data/bot.db"
    }
}

// Table migration
table users {
    id int primary key auto_increment
    name string not_null
    email string unique
    created_at string default "datetime('now')"
    index idx_email(email) unique
}

// Migration
migration add_role_column {
    table "users"
    add role_id int references roles(id)
}

// Model
model Profile "profiles" {
    user_id int
    bio string
    avatar string optional
}

// Queries
let user = db.query("SELECT * FROM users WHERE id = ?", [user_id])
db.execute("INSERT INTO users (name, email) VALUES (?, ?)", [name, email])
```

### Threading & Parallelism

```
cmd thread_demo {
    slash true
    parallel {
        thread {
            std.sleep(1)
            log info "Task 1 completed"
        }
        thread {
            std.sleep(2)
            log info "Task 2 completed"
        }
        thread {
            std.sleep(3)
            log info "Task 3 completed"
        }
    }
    reply "All tasks completed!"
}
```

### Modules & Imports

```
// Import entire module
import "modules/utility.ds" as util
util.help()

// Import specific items
import "modules/admin.ds" use ban, kick
ban(user)
kick(user)
```

### Standard Library

| Module | Functions |
|---|---|
| `std.strings` | `upper`, `lower`, `trim`, `split`, `join`, `replace`, `substring`, `length`, `contains`, `starts_with`, `ends_with`, `pad_left`, `pad_right`, `reverse`, `to_string` |
| `std.math` | `abs`, `min`, `max`, `floor`, `ceil`, `round`, `sqrt`, `pow`, `random`, `clamp`, `lerp` |
| `std.collections` | `length`, `is_empty`, `contains`, `sort`, `reverse`, `map`, `filter`, `reduce`, `find`, `some`, `every`, `keys`, `values`, `merge`, `slice`, `chunk`, `shuffle` |
| `std.time` | `now`, `format`, `parse`, `timestamp`, `sleep`, `duration`, `is_expired`, `remaining` |
| `std.discord` | `kick`, `ban`, `unban`, `clear_messages`, `create_role`, `delete_role`, `create_channel`, `delete_channel`, `create_thread`, `add_role`, `remove_role`, `send_dm`, `move_member` |

### Built-in Modules

| Module | Functions |
|---|---|
| `json` | `get(key)`, `set(key, value)`, `delete(key)`, `parse(string)`, `stringify(value)`, `push(key, value)`, `exists(key)`, `keys()`, `clear()` |
| `cache` | `get(key)`, `set(key, value, ttl_secs)`, `delete(key)`, `clear()`, `stats()` |
| `http` | `get(url, headers?)`, `post(url, body, headers?)`, `put(url, body, headers?)`, `delete(url, headers?)` |
| `db` | `query(sql, params?)`, `execute(sql, params?)`, `insert(sql, params?)`, `transaction(fn)` |

### Inline Tests

```
test "test_math_basics" {
    mock interaction {
        user: test_user
    }
    assert 1 + 1 == 2
    assert "hello" | upper == "HELLO"
    assert [1, 2, 3] | length == 3
}

test "test_list_operations" {
    let items = [5, 3, 8, 1]
    assert items | min == 1
    assert items | max == 8
    assert items | includes(3) == true
    assert items | includes(10) == false
}
```

---

## CLI Reference

```
discordscript init [name]         Create a new project
discordscript new <kind> <name>   Create a component (command|module|schedule|event|type)
discordscript run <file>          Run a .ds file (interpreter)
discordscript build <file>        Transpile to Python
discordscript check <file>        Static analysis
discordscript test <file>         Run inline tests
discordscript fmt <path>          Format source code
discordscript docs <file>         Generate documentation
discordscript env                 Validate .env variables
discordscript module <name>       Initialize a module
discordscript refcard <file>      Generate command reference
```

### `check` options

| Flag | Description |
|---|---|
| `--fix` | Auto-fix some issues (e.g., file naming) |
| `--strict` | Treat warnings as errors |

### `run` options

| Flag | Description |
|---|---|
| `--watch` | Hot-reload on file changes |
| `--token <token>` | Bot token (overrides .env) |
| `--log-level <level>` | Log level: debug, info, warn, error |

### `build` options

| Flag | Description |
|---|---|
| `--target <name>` | Backend: `discordpy`, `nextcord`, `pycord`, `discloud` |
| `--all` | Build for all backends |
| `--out <dir>` | Output directory |
| `--minify` | Minify generated Python |

---

## Project Structure

```
discordscript/
├── src/                      # Rust source code
│   ├── main.rs               # CLI entry point
│   ├── cli.rs                # Argument parsing (clap)
│   ├── lexer.rs              # Tokenizer
│   ├── parser.rs             # Syntax parser (~2900 lines)
│   ├── ast.rs                # AST types
│   ├── checker.rs            # Static analysis / linting
│   ├── formatter.rs          # Source code formatter
│   ├── runtime/
│   │   └── mod.rs            # Native interpreter
│   ├── stdlib/
│   │   └── mod.rs            # Standard library functions
│   ├── transpiler/
│   │   └── mod.rs            # Python code generation
│   └── modules/
│       └── mod.rs            # Built-in modules (json, cache, http, db)
├── examples/                 # Example .ds files
│   ├── hello.ds              # Simple ping/greet bot
│   ├── economy.ds            # Economy bot with DB & cooldowns
│   ├── moderation.ds         # Moderation bot with middleware
│   └── advanced.ds           # Full-featured demo (274 lines)
├── tests/
│   └── fixtures/             # 50+ .ds test files for manual testing
├── scripts/                  # Installation scripts
│   ├── install.sh            # Unix/macOS installer
│   └── install.ps1           # Windows installer
├── docs/                     # Documentation (mdBook)
│   ├── book.toml             # mdBook config
│   └── src/                  # Markdown source (20+ chapters)
│       ├── introduction.md
│       ├── getting-started.md
│       ├── contributing.md
│       ├── guide/            # Language guide (11 chapters)
│       ├── stdlib/           # Standard library reference
│       ├── modules/          # Built-in modules reference
│       ├── cli/              # CLI reference
│       └── examples/         # Example walkthroughs
├── build/                    # Transpiler output
│   ├── discordpy/
│   └── nextcord/
├── modules/                  # User-defined module directory
├── Cargo.toml                # Rust project config
└── README.md                 # This file
```

---

## Architecture

```
┌─────────────┐     ┌──────────┐     ┌───────────┐     ┌──────────────┐
│  .ds file   │────▶│  Lexer   │────▶│  Parser   │────▶│     AST      │
└─────────────┘     └──────────┘     └───────────┘     └──────┬───────┘
                                                              │
                    ┌─────────────────────────────────────────┤
                    │                                         │
                    ▼                                         ▼
           ┌────────────────┐                       ┌──────────────────┐
           │   Checker      │                       │   Transpiler     │
           │  (Static Ana.) │                       │  (Python Gen.)   │
           └───────┬────────┘                       └────────┬─────────┘
                   │                                         │
                   ▼                                         ▼
           ┌────────────────┐                       ┌──────────────────┐
           │   Runtime      │                       │  discord.py /    │
           │  (Interpreter) │                       │  nextcord / etc  │
           └────────────────┘                       └──────────────────┘
```

### Pipeline

1. **Lexer** — Tokenizes `.ds` source into tokens (keywords, identifiers, strings, numbers, operators, etc.)
2. **Parser** — Builds an AST (Abstract Syntax Tree) from tokens, handling all language constructs
3. **Checker** — Performs static analysis: validates intents, cron expressions, middleware references, permissions, cooldowns, path traversal, type consistency
4. **Runtime** — Executes the AST natively in a sandboxed environment with Discord API integration
5. **Transpiler** — Generates Python code for any supported Discord library backend
6. **Formatter** — Reconstructs source code from tokens with consistent styling

---

## Development

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Setup

```bash
git clone https://github.com/foksiny/discordscript
cd discordscript
cargo build
```

### Run tests

```bash
cargo test              # Run all tests (67+ tests)
cargo test lexer::      # Lexer tests only
cargo test parser::     # Parser tests only
cargo test checker::    # Checker tests only
cargo test runtime::    # Runtime tests only
```

### Manual testing

```bash
cargo run -- check examples/hello.ds
cargo run -- fmt examples/hello.ds
cargo run -- test examples/hello.ds
cargo run -- build examples/hello.ds --all
cargo run -- run examples/hello.ds --watch
```

### Build documentation

```bash
mdbook build docs
# Or serve with hot-reload:
mdbook serve docs --open
```

---

## Transpilation

DiscordScript can transpile `.ds` files to Python code for multiple Discord library backends:

| Backend | Command | Target |
|---|---|---|
| discord.py | `discordscript build bot.ds --target discordpy` | `build/discordpy/` |
| nextcord | `discordscript build bot.ds --target nextcord` | `build/nextcord/` |
| py-cord | `discordscript build bot.ds --target pycord` | `build/pycord/` |
| discloud | `discordscript build bot.ds --target discloud` | `build/discloud/` |
| All | `discordscript build bot.ds --all` | `build/*/` |

```bash
# Build for production
discordscript build bot.ds --target discordpy --out deploy/

# Install dependencies
cd deploy
pip install -r requirements.txt

# Run the bot
python main.py
```

---

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for any new functionality
4. Ensure all existing tests pass
5. Submit a PR with a clear description

See `docs/src/contributing.md` or the [Contributing Guide](https://github.com/foksiny/discordscript/blob/main/docs/src/contributing.md) for details.

---

## License

DiscordScript is licensed under the MIT License. See `LICENSE` for details.
