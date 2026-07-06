# Time Functions

## `std.now()`

Get the current Unix timestamp in milliseconds.

```
let start = std.now()
# ... do something ...
let elapsed = std.now() - start
std.log("Took {elapsed}ms")
```

## `std.sleep(milliseconds)`

Pause execution for a duration.

```
std.sleep(1000)  # wait 1 second
std.sleep(5000)  # wait 5 seconds
```

Useful for rate limiting:

```
let retries = 0
while retries < 3 {
    let result = try {
        http.get("https://api.example.com/data")
    } catch err {
        null
    }
    if result != null {
        break
    }
    std.sleep(1000)
    set retries = retries + 1
}
```

## Duration Operations

Timestamps support arithmetic:

```
let start = std.now()
std.sleep(500)
let end = std.now()
let elapsed = end - start  # ~500
```

## Time Formatting

Timestamps can be converted to readable formats via string interpolation:

```
let now = std.now()
std.log("Current time: {now}")  # prints the timestamp
```
