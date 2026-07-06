# DiscordScript

**DiscordScript** is a high-level, statically-analyzed scripting language designed specifically for building Discord bots. It compiles to multiple backends (discord.py, nextcord, py-cord, discloud) and can also run directly via its built-in interpreter.

## Why DiscordScript?

- **Write once, deploy anywhere** — transpile to any major Discord library
- **Concise syntax** — expressive without boilerplate
- **Safe by default** — static analysis catches issues before runtime
- **Batteries included** — built-in modules for HTTP, database, JSON, caching
- **Hot-reload ready** — edit scripts without restarting your bot

## Quick Tour

```
config {
    prefix "!"
    status "playing" "DiscordScript"
}

cmd ping {
    slash true
    description "Check if the bot is alive"
    reply "Pong!"
}

on ready {
    std.log("Bot is online!")
}
```

## Feature Highlights

| Feature | Description |
|---------|-------------|
| Commands | Slash, prefix, or hybrid commands with params, cooldowns, permissions |
| Events | Respond to any Discord gateway event |
| Schedules | Cron and interval-based task scheduling |
| Middleware | Reusable before/after hooks for commands |
| Modules | Split your bot into importable files |
| Embeds | Rich embedded messages with fields, images, footers |
| Components | Buttons, select menus, modals |
| Database | Built-in SQLite support via `db.*` |
| HTTP | Make API requests via `http.*` |
| JSON | In-memory key-value store |
| Cache | TTL-based caching layer |
| Threading | Parallel execution and async/await support |
| Testing | Built-in test blocks with assertions |
| Transpilation | Export to Python (discord.py, nextcord, py-cord, discloud) |
