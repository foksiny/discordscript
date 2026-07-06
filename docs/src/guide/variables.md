# Variables & Types

## Declaring Variables

Use `let` to declare a variable:

```
let name = "Alice"
let count = 42
let price = 19.99
let active = true
let data = null
```

Variables are immutable by default. Use `set` to reassign:

```
let x = 10
set x = x + 5
```

## Data Types

### Primitive Types

| Type | Examples | Description |
|------|----------|-------------|
| `String` | `"hello"` | Text data |
| `Integer` | `42`, `-10` | 64-bit signed integer |
| `Float` | `3.14`, `-0.5` | 64-bit floating point |
| `Boolean` | `true`, `false` | Logical value |
| `Null` | `null` | Absence of value |

### Collection Types

```
let list = [1, 2, 3]
let obj = { key: "value", count: 42 }
```

### Discord Types

| Type | Description |
|------|-------------|
| `User` | Discord user (id + name) |
| `Channel` | Discord channel (id + name) |
| `Role` | Discord role (id + name) |
| `Member` | Guild member (id + name + roles) |
| `Embed` | Rich embed object |

These types are returned by certain operations and passed to event handlers.

## Type Coercion

DiscordScript performs limited automatic type coercion in specific contexts:

- Strings are automatically interpolated in string contexts
- Numbers coerce to strings in concatenation
- All types have a truthy/falsy value in conditionals

## Truthiness

| Value | Truthy? |
|-------|---------|
| `true` | Yes |
| `false` | No |
| Non-zero numbers | Yes |
| `0` | No |
| Non-empty strings | Yes |
| Empty string `""` | No |
| Non-empty collections | Yes |
| Empty collections | No |
| `null` | No |
| Discord types (User, Channel, etc.) | Yes |

## Constants

Some names are pre-defined as globals:

```
true    # Boolean true
false   # Boolean false
null    # Null value
```

## Reserved Keywords

These keywords cannot be used as variable names:

`cmd`, `on`, `schedule`, `middleware`, `menu`, `type`, `test`, `import`, `export`, `use`, `as`, `let`, `set`, `if`, `elif`, `else`, `for`, `in`, `while`, `reply`, `send`, `embed`, `row`, `modal`, `cancel`, `return`, `throw`, `try`, `catch`, `finally`, `log`, `assert`, `mock`, `config`, `prefix`, `slash`, `description`, `param`, `permission`, `cooldown`, `per`, `guild_only`, `default`, `optional`, `required`, `kind`, `cron`, `every`, `event`, `next`, `and`, `or`, `not`, `true`, `false`, `null`, `discord`, `std`, `http`, `db`, `json`, `cache`, `file`, `voice`, `webhook`, `paginate`, `enum`, `struct`, `env`
