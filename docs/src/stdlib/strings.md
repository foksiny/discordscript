# String Functions

All string functions are accessed via `std.<function>()`.

## `std.upper(s)`

Convert a string to uppercase.

```
std.upper("hello")  # "HELLO"
```

## `std.lower(s)`

Convert a string to lowercase.

```
std.lower("HELLO")  # "hello"
```

## `std.len(s)`

Get the length of a string.

```
std.len("hello")  # 5
```

## `std.slice(s, start, end)`

Extract a substring.

```
std.slice("hello", 1, 4)  # "ell"
```

Negative indices are supported:

```
std.slice("hello", -3, -1)  # "ll"
```

## `std.join(list, separator)`

Join a list of strings with a separator.

```
std.join(["a", "b", "c"], ", ")  # "a, b, c"
```

## `std.contains(s, substring)`

Check if a string contains a substring.

```
std.contains("hello world", "world")  # true
```

## `std.replace(s, from, to)`

Replace all occurrences of a substring.

```
std.replace("hello world", "world", "there")  # "hello there"
```

## `std.trim(s)`

Remove whitespace from both ends.

```
std.trim("  hello  ")  # "hello"
```

## `std.split(s, delimiter)`

Split a string by a delimiter.

```
std.split("a,b,c", ",")  # ["a", "b", "c"]
```
