# Discord Helper Functions

These functions provide Discord-specific utilities, accessible via `std.*()`.

## `std.role_mention(role)`

Get the mention string for a role.

```
let msg = "Welcome to {std.role_mention(role)}!"
```

## `std.channel_mention(channel)`

Get the mention string for a channel.

```
let msg = "Check out {std.channel_mention(channel)}"
```

## `std.user_mention(user)`

Get the mention string for a user.

```
let msg = "Hello {std.user_mention(user)}!"
```

## `std.has_permission(member, permission)`

Check if a member has a specific permission.

```
if std.has_permission(member, "administrator") {
    reply "You're an admin!"
}
```

## `std.format_date(timestamp)`

Format a Unix timestamp as a readable date.

```
let joined = std.format_date(member.joined_at)
reply "You joined on {joined}"
```

## `std.avatar_url(user)`

Get the avatar URL for a user.

```
let avatar = std.avatar_url(user)
embed {
    title user.name
    image avatar
}
```

## `std.guild_icon(guild)`

Get the guild icon URL.

```
let icon = std.guild_icon(guild)
```
