# Cache Module

The cache module provides a TTL-based in-memory cache. Accessed via `cache.*`.

## `cache.set(key, value, ttl?)`

Store a value in the cache with optional TTL (in seconds).

```
cache.set("user:{user.id}", user_data, 300)       # 5 minute TTL
cache.set("config", app_config)                     # No expiration
```

## `cache.get(key)`

Retrieve a cached value. Returns `null` if not found or expired.

```
let user = cache.get("user:{user.id}")
if user == null {
    set user = db.find("users", { id: user.id })
    cache.set("user:{user.id}", user, 300)
}
reply "Hello {user.name}!"
```

## `cache.delete(key)`

Remove a value from the cache.

```
cache.delete("user:{user.id}")
```

## `cache.clear()`

Clear all cached values.

```
cache.clear()
```

## `cache.stats()`

Get cache statistics.

```
let stats = cache.stats()
std.log("Cache size: {stats.size} items")
```

## Cache-Aside Pattern

```
function get_user(id) {
    let cached = cache.get("user:{id}")
    if cached != null {
        return cached
    }
    let user = db.find("users", { id: id })
    if user != null {
        cache.set("user:{id}", user, 300)
    }
    return user
}
```

## Configuration

```
config {
    cache_ttl 300           # Default TTL in seconds
    cache_max_size "1GB"    # Maximum cache size
}
```
