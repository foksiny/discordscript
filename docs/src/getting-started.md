# Getting Started

## Installation

### Quick Install (One-liner)

**Linux / macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/foksiny/discordscript/stable/scripts/install.sh | bash
```

For a local (user-only) install:
```bash
curl -fsSL https://raw.githubusercontent.com/foksiny/discordscript/stable/scripts/install.sh | bash -s -- --local
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://raw.githubusercontent.com/foksiny/discordscript/stable/scripts/install.ps1 | iex"
```

**Via cargo (all platforms):**
```bash
cargo install --git https://github.com/foksiny/discordscript --branch stable
```

### From Source

```bash
git clone -b stable https://github.com/foksiny/discordscript
cd discordscript
cargo build --release
```

The binary will be at `target/release/discordscript.exe` (Windows) or `target/release/discordscript` (Linux/macOS).

### Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/foksiny/discordscript/releases).

## Your First Bot

### 1. Create a project

```bash
discordscript init my-bot
cd my-bot
```

This creates the following structure:

```
my-bot/
├── main.ds          # Main bot file
├── modules/         # Module files
├── events/          # Event files
├── schedules/       # Schedule files
├── .env.example     # Environment template
└── README.md        # Project README
```

### 2. Write your first command

Open `main.ds`:

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

### 3. Check for errors

```bash
discordscript check main.ds
```

### 4. Run the bot

```bash
discordscript run main.ds
```

### 5. Build for deployment

```bash
# Build for a specific backend
discordscript build main.ds --target discordpy

# Build for all backends
discordscript build main.ds --all
```

## Next Steps

- Learn the [syntax basics](guide/syntax.md)
- Explore [commands](guide/commands.md) in depth
- Check out the [examples](examples/hello.md)
