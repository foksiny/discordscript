# Economy Bot Example

A simple economy system with balance, daily rewards, and a leaderboard.

```discordscript
config {
    prefix "!"
    status "watching" "economy"
    intents ["guilds", "messages"]
    db "sqlite://data/economy.db"
}

cmd balance {
    slash true
    description "Check your coin balance"

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
    description "Claim your daily reward"
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

cmd leaderboard {
    slash true
    description "Show top coin holders"

    let top_users = db.find_all("users", {})
    let count = std.len(top_users)
    if count > 0 {
        embed {
            title "Leaderboard"
            description "Top {count} coin holders"
            color "gold"
        }
    }
}
```
