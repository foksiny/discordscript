# Events

Events let you respond to Discord gateway events.

## Basic Event

```
on <event_name> {
    # event handler body
}
```

## Available Events

| Event | Fires When |
|-------|------------|
| `ready` | Bot has connected to Discord |
| `message` | A message is sent in a readable channel |
| `member_join` | A member joins the guild |
| `member_leave` | A member leaves the guild |
| `member_update` | A member's profile changes |
| `message_delete` | A message is deleted |
| `message_update` | A message is edited |
| `reaction_add` | A reaction is added |
| `reaction_remove` | A reaction is removed |
| `voice_join` | A member joins a voice channel |
| `voice_leave` | A member leaves a voice channel |
| `guild_join` | Bot joins a new guild |
| `guild_leave` | Bot leaves a guild |
| `interaction` | Any interaction is received |

## Event Variables

Each event provides built-in variables:

```
on member_join {
    # member — the joining member
    # guild — the guild
    reply "Welcome {member.name} to {guild.name}!"
}

on message {
    # message — the message object
    # author — the message author
    # channel — the channel
    if message.content == "ping" {
        reply "pong!"
    }
}
```

## Using Events for Logging

```
on member_join {
    let channel = discord.get_channel("welcome")
    send "Welcome {member.mention}!" in channel channel
}

on member_leave {
    let channel = discord.get_channel("logs")
    send "{member.name} left the server." in channel channel
}
```

## Multiple Events

You can define as many event handlers as you need:

```
on ready {
    std.log("Bot is online!")
    discord.set_status("watching", "{guild_count} servers")
}

on guild_join {
    std.log("Joined new guild: {guild.name}")
}

on guild_leave {
    std.log("Left guild: {guild.name}")
}
```
