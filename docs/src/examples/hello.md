# Hello World Example

A minimal DiscordScript bot.

```discordscript
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

cmd hello {
    slash true
    param name {
        type string
        description "Who to greet"
        required false
        default "World"
    }
    reply "Hello, {name}!"
}

on ready {
    std.log("Bot is online!")
}
```

## Running

```bash
discordscript check main.ds
discordscript run main.ds
```

## Check Output

```
File: main.ds
Lines: 30
Commands: 2
Events: 1
No issues found.
```
