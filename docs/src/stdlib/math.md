# Math Functions

## `std.random(min, max)`

Generate a random integer between min and max (inclusive).

```
let dice = std.random(1, 6)
```

## `std.clamp(value, min, max)`

Clamp a value between min and max.

```
std.clamp(150, 0, 100)  # 100
std.clamp(-5, 0, 100)   # 0
std.clamp(50, 0, 100)   # 50
```

## `std.abs(n)`

Absolute value.

```
std.abs(-42)  # 42
```

## `std.min(a, b)`

Return the smaller of two numbers.

```
std.min(10, 20)  # 10
```

## `std.max(a, b)`

Return the larger of two numbers.

```
std.max(10, 20)  # 20
```

## `std.round(n)`

Round to nearest integer.

```
std.round(3.14)  # 3
std.round(3.75)  # 4
```

## `std.floor(n)`

Round down.

```
std.floor(3.99)  # 3
```

## `std.ceil(n)`

Round up.

```
std.ceil(3.01)  # 4
```
