# Middleware

Middleware provides reusable before/after hooks for commands. They're like decorators that wrap command execution.

## Defining Middleware

```
middleware <name> {
    # runs before the command
    next    # calls the next middleware or the command
    # runs after the command
}
```

The `next` statement passes control to the next middleware or the command itself. Code before `next` runs before the command, code after `next` runs after.

## Example Middleware

### Logging Middleware

```
middleware logging {
    let start = std.now()
    std.log("Command started by {user.tag}")
    next
    let elapsed = std.now() - start
    std.log("Command finished in {elapsed}ms")
}
```

### Permission Check

```
middleware check_admin {
    if !user.has_permission("administrator") {
        reply "You don't have permission to use this command!" ephemeral true
        cancel
    }
    next
}
```

### Rate Limiting

```
middleware rate_limit {
    let key = "ratelimit:{user.id}"
    let count = cache.get(key)
    if count != null && count >= 5 {
        reply "Too many requests! Please wait." ephemeral true
        cancel
    }
    cache.set(key, (count != null ? count + 1 : 1), 60)
    next
}
```

## Using Middleware

Apply middleware to commands:

```
cmd admin_panel {
    slash true
    permission "administrator"
    middleware ["check_admin", "logging"]

    # command body
    reply "Welcome to the admin panel!"
}
```

Middleware runs in the order they're listed.

## Global Middleware

Apply middleware to all commands via config:

```
config {
    global_middleware ["logging", "rate_limit"]
}
```

## Middleware with Parameters

Middleware can access all command context:

```
middleware audit {
    next
    # Log after command
    let channel = discord.get_channel("audit-log")
    send "{user.tag} used {command.name}" in channel channel
}
```
