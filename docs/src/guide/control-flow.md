# Control Flow

## Conditionals

### If / Elif / Else

```
if condition {
    # runs when condition is truthy
}

if condition {
    # truthy branch
} else {
    # falsy branch
}

if condition1 {
    # first branch
} elif condition2 {
    # second branch
} else {
    # fallback
}
```

Conditions are expressions that evaluate to any value. The [truthiness rules](variables.md#truthiness) determine whether a branch executes.

```
let score = 85

if score >= 90 {
    reply "Grade: A"
} elif score >= 80 {
    reply "Grade: B"
} elif score >= 70 {
    reply "Grade: C"
} else {
    reply "Grade: F"
}
```

### Inline If (ternary)

For simple conditions, use the `if` expression inline:

```
let status = if age >= 18 { "adult" } else { "minor" }
```

## Loops

### For Loop

Iterate over collections:

```
let items = ["apple", "banana", "cherry"]
for item in items {
    std.log("Fruit: {item}")
}
```

Iterate over a range:

```
for i in [1, 2, 3, 4, 5] {
    std.log("Count: {i}")
}
```

### While Loop

```
let count = 0
while count < 5 {
    std.log("Count: {count}")
    set count = count + 1
}
```

## Pattern Matching

DiscordScript supports pattern matching for destructuring:

```
let [first, second] = list
let { name, age } = user_data
```

## Short-circuit Evaluation

Logical operators short-circuit:

```
if user != null && user.has_permission("admin") {
    # user is guaranteed non-null here
}
```
