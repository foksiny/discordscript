# Threading & Parallelism

## Threads

Run code in a separate thread:

```
thread {
    let result = http.get("https://api.example.com/data")
    cache.set("api_data", result)
}
```

Threads run concurrently. Use `await` to wait for a result:

```
let future = thread {
    http.get("https://api.example.com/data")
}
let result = await future
```

## Parallel Execution

Run multiple operations simultaneously:

```
parallel {
    branch {
        let users = db.find_all("users")
    }
    branch {
        let posts = db.find_all("posts")
    }
    branch {
        let stats = cache.get("stats")
    }
}
```

Each `branch` runs in parallel. You can capture results:

```
parallel result_var results {
    branch {
        db.find_all("users")
    }
    branch {
        db.find_all("posts")
    }
}
# results[0] = users list, results[1] = posts list
```

## Async / Await

```
let user_data = await http.get("https://api.example.com/user/123")
let posts = await http.get("https://api.example.com/user/123/posts")
reply "Found {posts.len()} posts for {user_data.name}"
```

## Thread Safety

Variables are shared across threads via an Arc\<Mutex\>. Mutations are synchronized:

```
let counter = 0
parallel {
    branch {
        for i in [1, 2, 3] {
            set counter = counter + 1
        }
    }
    branch {
        for i in [1, 2, 3] {
            set counter = counter + 1
        }
    }
}
# counter will be 6
```

## Best Practices

- Use `thread` for I/O-bound operations (HTTP, database)
- Use `parallel` for independent operations
- Avoid sharing mutable state across branches when possible
- Use `await` to coordinate parallel work
- Don't create more threads than available cores (configurable via `config { threads N }`)
