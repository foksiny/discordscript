# Advanced Bot Example

A comprehensive example showing advanced DiscordScript features.

You can find the full source at `examples/advanced.ds`.

## Commands

| Command | Description |
|---------|-------------|
| `ping` | Basic ping-pong |
| `info` | Bot information with embed |
| `greet` | User greeting with embed |
| `echo` | Echo with button components |
| `poll` | Poll with select menu |
| `feedback` | Modal input form |
| `database` | Database CRUD demo |
| `http_demo` | HTTP request demo |
| `cache_demo` | Cache operations demo |
| `embed_demo` | Rich embed showcase |
| `row_demo` | Button row demo |
| `paginate` | Paginated list |
| `json_store` | JSON key-value store |

## Events

| Event | Handler |
|-------|---------|
| `ready` | Logs bot startup |
| `member_join` | Welcome message |

## Middleware

| Middleware | Purpose |
|------------|---------|
| `logging` | Logs command usage |
| `check_admin` | Permission check |

## Schedules

| Schedule | When | Action |
|----------|------|--------|
| `daily_cleanup` | Every 24h | Cleanup task |
| `status_update` | Every hour | Update bot status |

## Running

```bash
discordscript check examples/advanced.ds
discordscript run examples/advanced.ds
```
