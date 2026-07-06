# Project Setup

## Structure

A typical DiscordScript project looks like:

```
my-bot/
├── main.ds              # Entry point, commands, config
├── modules/             # Module files
│   ├── admin.ds
│   └── utility.ds
├── events/              # Event handlers
│   ├── welcome.ds
│   └── logging.ds
├── schedules/           # Scheduled tasks
│   ├── backup.ds
│   └── stats.ds
├── .env                 # Environment variables (git-ignored)
├── .env.example         # Environment template
├── .gitignore
└── README.md
```

## Quick Start

```bash
# Create a project
discordscript init my-bot
cd my-bot

# Add a command
discordscript new command ping

# Add an event handler
discordscript new event ready

# Add a schedule
discordscript new schedule hourly

# Check for errors
discordscript check main.ds

# Run the bot
discordscript run main.ds
```

## Configuration

The `config` block in your entry file controls bot behavior:

```
config {
    prefix "!"                          # Default prefix for prefix commands
    status "playing" "DiscordScript"    # Bot status
    intents ["guilds", "messages"]      # Gateway intents
    threads 4                           # Thread pool size
    db "sqlite://data/bot.db"          # Database URL
    log_level "info"                    # Log level (debug, info, warn, error)
    log_file "logs/bot.log"            # Log file path
    cache_ttl 300                       # Default cache TTL in seconds
    error_channel "123456789"          # Channel ID for error reports
    error_ephemeral true                # Send errors as ephemeral messages
    global_middleware ["logging"]       # Middleware applied to all commands
}
```

## Environment Variables

Create a `.env` file for sensitive data:

```
DISCORD_TOKEN=your_bot_token_here
DATABASE_URL=sqlite://data/prod.db
API_KEY=your_api_key
```

Access via `env.VARIABLE_NAME`:

```
config {
    db env.DATABASE_URL
}
```

## .gitignore

```
node_modules/
target/
.env
build/
logs/
*.log
```

## Building for Production

```bash
# Build for a specific target
discordscript build main.ds --target discordpy

# Build all targets
discordscript build main.ds --all --out ./dist
```
