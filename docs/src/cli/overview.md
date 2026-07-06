# CLI Reference

## Usage

```
discordscript <COMMAND> [OPTIONS] [ARGS]
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Create a new DiscordScript project |
| `new` | Generate a new command, event, schedule, or type |
| `run` | Run a DiscordScript file (interpreter mode) |
| `check` | Static analysis without execution |
| `build` | Transpile to a target backend |
| `fmt` | Format source files |
| `test` | Run test blocks |
| `docs` | Generate documentation |
| `module` | Create a new module file |
| `env` | Display environment variables |

## Global Options

| Option | Description |
|--------|-------------|
| `--help` | Show help information |
| `--version` | Show version information |

## `discordscript init`

```
discordscript init [name]
```

Creates a new DiscordScript project with default structure. If `name` is omitted, uses `my-bot`.

## `discordscript new`

```
discordscript new <command|module|schedule|event|type> <name>
```

Generates a boilerplate file of the given type.

## `discordscript run`

```
discordscript run <file> [--watch]
```

Executes a DiscordScript file. With `--watch`, enables hot-reload mode.

## `discordscript check`

```
discordscript check <file> [--fix] [--strict]
```

Runs static analysis (duplicate names, missing triggers, invalid config, etc.). With `--fix`, auto-fixes some issues.

## `discordscript build`

```
discordscript build <file> --target <discordpy|nextcord|pycord|discloud>
discordscript build <file> --all
```

Transpiles to Python. Use `--all` to generate all backends.

## `discordscript fmt`

```
discordscript fmt <path> [--check]
```

Re-formats source files. With `--check`, only checks formatting (exits with code 1 if not formatted).

## `discordscript test`

```
discordscript test <file> [--verbose]
```

Runs `test` blocks defined in the file. Returns non-zero exit code if any test fails.

## `discordscript docs`

```
discordscript docs <file> [--serve] [--only <name>]
```

Generates markdown documentation. With `--serve`, starts a web server.

## `discordscript module`

```
discordscript module <name>
```

Creates a new module file in the `modules/` directory.

## `discordscript env`

```
discordscript env [--show]
```

Displays environment variables from `.env` file. Without `--show`, values are masked.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error / test failure |
