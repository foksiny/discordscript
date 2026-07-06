# Error Handling

## Try/Catch

```
try {
    let result = db.query("INSERT INTO users VALUES (?, ?)", [name, age])
    reply "User created!"
} catch err {
    reply "Failed to create user: {err}" ephemeral true
} finally {
    std.log("Database operation attempted")
}
```

## Throw

Raise custom errors:

```
if user == null {
    throw "User not found"
}
```

## Return

Exit early from a command or handler:

```
cmd secret {
    slash true
    permission "admin"
    if !user.has_permission("admin") {
        reply "Access denied!" ephemeral true
        return
    }
    reply "Secret data"
}
```

## Cancel

Stop middleware chain or command execution:

```
middleware auth {
    if !user.is_authenticated {
        reply "Please log in first" ephemeral true
        cancel
    }
    next
}
```

## Assert

For testing and validation:

```
test "database operations" {
    let result = db.insert("users", { name: "test" })
    assert result != null, "Insert should return a result"
}
```

## Error Handling Patterns

### Graceful Fallbacks

```
let data = try {
    http.get("https://api.example.com/data")
} catch err {
    std.log("API error: {err}")
    null
}
```

### Validation

```
param email {
    type string
    required true
}
if !email.contains("@") {
    reply "Invalid email address" ephemeral true
    return
}
```
