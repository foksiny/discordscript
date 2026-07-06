# HTTP Module

The HTTP module lets you make HTTP/HTTPS requests. Accessed via `http.*`.

## `http.get(url)`

Make a GET request.

```
let response = http.get("https://api.github.com/users/octocat")
std.log("Status: {response.status}")
```

## `http.post(url, body)`

Make a POST request.

```
let response = http.post("https://api.example.com/data", {
    name: "Test",
    value: 42
})
```

## `http.put(url, body)`

Make a PUT request.

```
http.put("https://api.example.com/data/1", { name: "Updated" })
```

## `http.delete(url)`

Make a DELETE request.

```
http.delete("https://api.example.com/data/1")
```

## `http.stream(url)`

Open a streaming connection.

```
http.stream("https://api.example.com/events")
```

## Response Format

HTTP responses return an object with:

```
{
    status: 200,                    # Status code
    body: "...",                    # Response body (string)
    headers: { ... }                # Response headers
}
```

## Configuration

HTTP settings can be configured in `config`:

```
config {
    http_timeout 30          # Timeout in seconds
    http_retry 3             # Number of retries on failure
}
```
