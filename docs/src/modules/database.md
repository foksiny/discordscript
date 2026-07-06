# Database Module

The database module provides SQLite database access. Accessed via `db.*`.

## Configuration

```
config {
    db "sqlite://data/bot.db"
    db_auto_migrate true
}
```

## `db.query(sql, params)`

Execute a raw SQL query.

```
let users = db.query("SELECT * FROM users WHERE active = ?", [true])
```

## `db.find(collection, query)`

Find a single record.

```
let user = db.find("users", { id: user_id })
```

## `db.find_all(collection, query)`

Find all matching records.

```
let all_users = db.find_all("users", {})
let active_users = db.find_all("users", { active: true })
```

## `db.insert(collection, data)`

Insert a new record.

```
let new_user = db.insert("users", {
    name: "Alice",
    score: 100,
    active: true
})
```

## `db.update(collection, query, data)`

Update matching records.

```
db.update("users", { id: user_id }, { score: 200 })
```

## `db.delete(collection, query)`

Delete matching records.

```
db.delete("users", { id: user_id })
```

## Example: Economy System

```
cmd balance {
    slash true
    let user_data = db.find("users", { id: user.id })
    if user_data == null {
        db.insert("users", { id: user.id, balance: 0 })
        reply "Welcome! Your balance is 0 coins."
    } else {
        reply "Your balance: {user_data.balance} coins"
    }
}

cmd daily {
    slash true
    cooldown 86400 "s" per user
    let user_data = db.find("users", { id: user.id })
    if user_data == null {
        db.insert("users", { id: user.id, balance: 100 })
    } else {
        db.update("users", { id: user.id }, {
            balance: user_data.balance + 100
        })
    }
    reply "You received 100 coins!"
}
```
