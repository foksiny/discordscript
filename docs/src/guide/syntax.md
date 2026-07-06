# Syntax Basics

DiscordScript uses a brace-delimited, expression-oriented syntax inspired by Rust and Go.

## Comments

```
# This is a single-line comment
```

Comments start with `#` and extend to the end of the line. Multi-line comments are not supported — use multiple `#` lines.

## Literals

```
"hello world"        # String
"interpolated {x}"   # Interpolated string
42                   # Integer
3.14                 # Float
true                 # Boolean
false                # Boolean
null                 # Null value
```

## Strings

Plain strings use double quotes:

```
let name = "Alice"
```

Interpolated strings evaluate expressions inside `{}`:

```
let greeting = "Hello, {name}!"
let message = "2 + 2 = {2 + 2}"
```

Escaping inside strings uses backslash:

```
let text = "Line 1\nLine 2"
let quote = "She said \"hello\""
```

## Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```
my_variable   _count   user123   PREFIX
```

Some keywords cannot be used as identifiers (see [Variables & Types](variables.md) for the full list).

## Operators

### Arithmetic

| Operator | Description |
|----------|-------------|
| `+` | Addition / string concatenation |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `%` | Modulo |

### Comparison

| Operator | Description |
|----------|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less than or equal |
| `>` | Greater than |
| `>=` | Greater than or equal |

### Logical

| Operator | Description |
|----------|-------------|
| `&&` | Logical AND |
| `\|\|` | Logical OR |
| `!` | Logical NOT (prefix) |

### Assignment

| Operator | Description |
|----------|-------------|
| `=` | Assignment |

## Blocks

Blocks are delimited by curly braces `{ }` and can contain multiple statements:

```
{
    let x = 1
    let y = 2
    set x = x + y
}
```

## Statements

Each statement goes on its own line. Statements are not terminated by semicolons (though semicolons are allowed).

```
let name = "Alice"       # Variable declaration
set name = "Bob"         # Variable reassignment
reply "Hello!"           # Discord reply
std.log("message")       # Standard library call
if condition { ... }     # Conditional
for item in list { ... } # Loop
```
