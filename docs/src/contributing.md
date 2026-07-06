# Contributing

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Git

### Clone & Build

```bash
git clone https://github.com/foksiny/discordscript
cd discordscript
cargo build
```

### Run Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs          # CLI entry point & commands
├── cli.rs           # Clap argument parsing
├── lexer.rs         # Tokenizer
├── parser.rs        # Syntax parser
├── ast.rs           # Abstract syntax tree types
├── checker.rs       # Static analysis
├── runtime/
│   └── mod.rs       # Interpreter
├── transpiler/
│   └── mod.rs       # Python code generation
├── modules/
│   └── mod.rs       # Built-in module implementations
├── stdlib/
│   └── mod.rs       # Standard library functions
├── formatter.rs     # Source code formatter
docs/                # Website documentation (mdBook)
examples/            # Example DiscordScript files
```

## Style Guide

- 4-space indentation
- `snake_case` for variable and function names
- Descriptive variable names
- Comments for non-obvious logic

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Write tests for any new functionality
4. Ensure all existing tests pass
5. Submit a PR with a clear description

## License

DiscordScript is licensed under the MIT License.
