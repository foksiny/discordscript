# JSON Module

The JSON module provides an in-memory key-value store. Accessed via `json.*`.

## `json.set(key, value)`

Set a value by key.

```
json.set("user_count", 42)
json.set("greeting", "Hello World")
json.set("config", { theme: "dark", lang: "en" })
```

## `json.get(key)`

Get a value by key. Returns `null` if not found.

```
let count = json.get("user_count")      # 42
let missing = json.get("nonexistent")   # null
```

## `json.delete(key)`

Delete a key-value pair.

```
json.delete("temporary_data")
```

## `json.push(key, value)`

Push a value to a list stored at key. Creates the list if it doesn't exist.

```
json.push("logs", "Server started")
json.push("logs", "User joined")
json.push("logs", "Command executed")
let logs = json.get("logs")  # ["Server started", "User joined", "Command executed"]
```

## `json.save()`

Persist data to disk (stub — in-memory for now).

```
json.save()
```

## Use Cases

### Temporary State

```
cmd set_lang {
    slash true
    param lang { type string }
    json.set("lang:{user.id}", lang)
    reply "Language set to {lang}"
}

cmd greet {
    slash true
    let lang = json.get("lang:{user.id}")
    if lang == "pt" {
        reply "Olá!"
    } else {
        reply "Hello!"
    }
}
```

### Caching Config

```
let config = json.get("app_config")
if config == null {
    set config = db.find("config", { id: 1 })
    json.set("app_config", config)
}
```
