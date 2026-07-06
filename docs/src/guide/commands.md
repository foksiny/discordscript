# Commands

Commands are the core of any Discord bot. DiscordScript supports slash commands, prefix commands, and hybrid commands.

## Basic Command

```
cmd <name> {
    slash true
    prefix "!"
    description "What this command does"

    reply "Hello!"
}
```

## Command Properties

### Triggers

```
cmd greet {
    slash true                              # Slash command
    prefix "!"                              # Prefix command ("!greet")
    prefix true                              # Prefix with default prefix from config
}
```

### Description & Permissions

```
cmd admin {
    slash true
    description "Admin-only command"
    permission "administrator"
    guild_only true
}
```

### Cooldown

```
cmd daily {
    slash true
    cooldown 86400 "s" per user
    reply "Here's your daily reward!"
}
```

Parameters: `cooldown <seconds> "<unit>" [per <scope>]`

Valid scopes: `user`, `channel`, `guild`, `global`

### Middleware

```
cmd secure {
    slash true
    middleware ["auth", "logging"]
    reply "Secure data"
}
```

Middleware runs before the command body. See [Middleware](middleware.md).

## Parameters

```
cmd greet {
    slash true
    param name {
        type string
        description "Name to greet"
        required true
    }
    param age {
        type int
        description "Age (optional)"
        required false
        default 0
    }
    reply "Hello, {name}! You are {age} years old."
}
```

### Parameter Types

| Type | Description |
|------|-------------|
| `string` | Text input |
| `int` | Integer number |
| `float` | Decimal number |
| `bool` | Boolean |
| `user` | Discord user mention |
| `channel` | Discord channel mention |
| `role` | Discord role mention |
| `member` | Guild member |
| `attachment` | File upload |

## Subcommands

```
cmd config {
    slash true
    sub set {
        param key { type string }
        param value { type string }
        reply "Set {key} = {value}"
    }
    sub get {
        param key { type string }
        reply "Value: {get_config(key)}"
    }
}
```

## Context Menus

```
menu user "User Info" {
    reply "User: {user.tag}\nID: {user.id}"
}

menu message "Translate" {
    reply "Message: {message.content}"
}
```

Use `menu user` for user context menus and `menu message` for message context menus.

## Command Body

The command body can contain any statements: replies, embeds, conditions, loops, function calls, etc.

```
cmd stats {
    slash true
    let uptime = std.now() - start_time
    embed {
        title "Bot Stats"
        field "Uptime" value "{uptime}s" inline true
        field "Commands" value "{cmd_count}" inline true
        color "green"
    }
}
```
