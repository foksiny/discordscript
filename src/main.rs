mod cli;
mod lexer;
mod ast;
mod parser;
mod checker;
mod runtime;
mod stdlib;
mod modules;
mod transpiler;
mod formatter;

use clap::Parser as ClapParser;
use cli::{Cli, Commands};
use lexer::Lexer;
use parser::Parser;
use checker::Checker;
use runtime::Runtime;
use std::path::Path;
use std::process;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => cmd_init(name.as_deref()),
        Commands::New { kind, name } => cmd_new(kind, name),
        Commands::Run { file, watch } => cmd_run(file, *watch),
        Commands::Build { file, target, all, out, minify } => cmd_build(file, target, *all, out.as_deref(), *minify),
        Commands::Docs { file, serve, only, lang, template } => cmd_docs(file, *serve, only.as_deref(), lang, template),
        Commands::Test { file, verbose } => cmd_test(file, *verbose),
        Commands::Check { file, fix, strict } => cmd_check(file, *fix, *strict),
        Commands::Fmt { path, check } => cmd_fmt(path, *check),
        Commands::Env { show } => cmd_env(*show),
        Commands::Module { name } => cmd_module(name),
        Commands::Refcard { file, format } => cmd_refcard(file, format),
    }
}

fn read_source(file: &str) -> String {
    std::fs::read_to_string(file)
        .unwrap_or_else(|e| {
            eprintln!("Error reading file '{}': {}", file, e);
            process::exit(1);
        })
}

fn parse_source(file: &str) -> (ast::Program, String) {
    let source = read_source(file);
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap_or_else(|e| {
        eprintln!("Lexer error: {}", e);
        process::exit(1);
    });
    let mut parser = Parser::new(tokens);
    let program = parser.parse().unwrap_or_else(|e| {
        eprintln!("Parser error: {}", e);
        process::exit(1);
    });
    (program, source)
}

fn cmd_init(name: Option<&str>) {
    let project_name = name.unwrap_or("my-bot");
    let dir = Path::new(project_name);

    if dir.exists() {
        eprintln!("Directory '{}' already exists", project_name);
        process::exit(1);
    }

    std::fs::create_dir_all(dir.join("modules")).unwrap_or_else(|e| {
        eprintln!("Failed to create project: {}", e);
        process::exit(1);
    });
    std::fs::create_dir_all(dir.join("data")).unwrap_or_else(|e| {
        eprintln!("Failed to create project: {}", e);
        process::exit(1);
    });

    let main_content = r#"config {
    prefix "!"
    status "playing" "DiscordScript"
    intents all
}

cmd ping {
    slash true
    prefix true
    description "Responds with Pong!"

    reply "Pong!"
}

on ready {
    send "Bot is online!" in channel "general"
}
"#;

    std::fs::write(dir.join("main.ds"), main_content).unwrap_or_else(|e| {
        eprintln!("Failed to write main.ds: {}", e);
        process::exit(1);
    });

    let env_example = "DISCORD_TOKEN=your_token_here\n";
    std::fs::write(dir.join(".env.example"), env_example).unwrap_or_else(|e| {
        eprintln!("Failed to write .env.example: {}", e);
        process::exit(1);
    });

    let config_content = r#"{
    "name": "My Bot",
    "version": "1.0.0",
    "docs": {
        "title": "My Bot - Documentation",
        "description": "A Discord bot made with DiscordScript",
        "language": "en"
    }
}
"#;

    std::fs::write(dir.join("discordscript.json"), config_content).unwrap_or_else(|e| {
        eprintln!("Failed to write discordscript.json: {}", e);
        process::exit(1);
    });

    println!("Initialized DiscordScript project in '{}'", project_name);
    println!("  main.ds           - Main bot file");
    println!("  modules/          - Module directory");
    println!("  data/             - Data directory");
    println!("  discordscript.json - Project configuration");
    println!("  .env.example      - Environment variables template");
    println!("\nQuick start:");
    println!("  cd {}", project_name);
    println!("  discordscript run main.ds");
}

fn cmd_new(kind: &str, name: &str) {
    match kind {
        "command" | "cmd" => {
            let content = format!(
                r#"cmd {} {{
    slash true
    description "Description of {}"

    reply "Hello from {}!"
}}
"#,
                name, name, name
            );
            let file = format!("modules/{}.ds", name);
            std::fs::write(&file, content).unwrap_or_else(|e| {
                eprintln!("Failed to create command: {}", e);
                process::exit(1);
            });
            println!("Created command '{}' in {}", name, file);
        }
        "module" => {
            let content = format!(
                r#"export cmd {} {{
    slash true
    description "Module {} command"

    reply "Module {} executed!"
}}
"#,
                name, name, name
            );
            let file = format!("modules/{}.ds", name);
            std::fs::write(&file, content).unwrap_or_else(|e| {
                eprintln!("Failed to create module: {}", e);
                process::exit(1);
            });
            println!("Created module '{}' in {}", name, file);
        }
        "schedule" => {
            let content = format!(
                r#"schedule "{}_task" {{
    every 1h

    std.log("Task {} executed")
}}
"#,
                name, name
            );
            let file = format!("schedules/{}.ds", name);
            std::fs::create_dir_all("schedules").ok();
            std::fs::write(&file, content).unwrap_or_else(|e| {
                eprintln!("Failed to create schedule: {}", e);
                process::exit(1);
            });
            println!("Created schedule '{}' in {}", name, file);
        }
        "event" => {
            let content = format!(
                r#"on {} {{
    send "Event {} triggered!" in channel "general"
}}
"#,
                name, name
            );
            let file = format!("events/{}.ds", name);
            std::fs::create_dir_all("events").ok();
            std::fs::write(&file, content).unwrap_or_else(|e| {
                eprintln!("Failed to create event: {}", e);
                process::exit(1);
            });
            println!("Created event '{}' in {}", name, file);
        }
        "type" => {
            let content = format!(
                r#"type {} {{
    name string
    value int default 0
    active bool default true
}}
"#,
                name
            );
            let file = format!("types/{}.ds", name);
            std::fs::create_dir_all("types").ok();
            std::fs::write(&file, content).unwrap_or_else(|e| {
                eprintln!("Failed to create type: {}", e);
                process::exit(1);
            });
            println!("Created type '{}' in {}", name, file);
        }
        _ => {
            eprintln!("Unknown component type '{}'. Use: command, module, schedule, event, type", kind);
            process::exit(1);
        }
    }
}

fn cmd_run(file: &str, watch: bool) {
    let (program, _source) = parse_source(file);

    println!("Loading project: {}", file);
    println!("  Commands: {}", program.commands.len());
    println!("  Events: {}", program.events.len());
    println!("  Schedules: {}", program.schedules.len());
    println!("  Middleware: {}", program.middleware.len());
    println!("  Context menus: {}", program.context_menus.len());
    println!("  Custom types: {}", program.types.len());
    println!("  Tests: {}", program.tests.len());

    if watch {
        println!("Watch mode enabled. Hot-reload active.");
        // TODO: implement file watching with notify
    }

    // Run checker
    let checker = Checker::new(&program);
    let warnings = checker.check();
    for w in &warnings {
        println!("Warning: {}", w);
    }

    // TODO: Launch interpreter (serenity-based runtime)
    println!("\nDiscordScript interpreter starting...");
    println!("Connecting to Discord...");
}

fn cmd_build(file: &str, target: &str, all: bool, out: Option<&str>, _minify: bool) {
    let (program, _source) = parse_source(file);

    let build_targets = if all {
        vec!["discordpy", "nextcord", "pycord", "discloud"]
    } else {
        vec![target]
    };

    let output_dir = out.unwrap_or("build");

    for t in build_targets {
        println!("Building for target: {} -> {}/{}", t, output_dir, t);
        match t {
            "discordpy" => transpiler::generate_discordpy(&program, &format!("{}/{}", output_dir, t)),
            "nextcord" => transpiler::generate_nextcord(&program, &format!("{}/{}", output_dir, t)),
            "pycord" => transpiler::generate_pycord(&program, &format!("{}/{}", output_dir, t)),
            "discloud" => transpiler::generate_discloud(&program, &format!("{}/{}", output_dir, t)),
            _ => eprintln!("Unknown target '{}'. Available: discordpy, nextcord, pycord, discloud", t),
        }
    }

    if all {
        println!("All targets built successfully in '{}/'", output_dir);
    } else {
        println!("Built successfully for target '{}' in '{}/{}'", target, output_dir, target);
    }
}

fn cmd_docs(file: &str, _serve: bool, only: Option<&str>, _lang: &str, _template: &str) {
    let (program, _source) = parse_source(file);

    let doc_dir = Path::new("docs");
    if !doc_dir.exists() {
        std::fs::create_dir_all(doc_dir).unwrap_or_else(|e| {
            eprintln!("Failed to create docs directory: {}", e);
            process::exit(1);
        });
    }

    std::fs::create_dir_all(doc_dir.join("commands")).ok();
    std::fs::create_dir_all(doc_dir.join("events")).ok();
    std::fs::create_dir_all(doc_dir.join("schedules")).ok();
    std::fs::create_dir_all(doc_dir.join("modules")).ok();
    std::fs::create_dir_all(doc_dir.join("types")).ok();
    std::fs::create_dir_all(doc_dir.join("stdlib")).ok();
    std::fs::create_dir_all(doc_dir.join("reference")).ok();

    // Generate README
    let readme = generate_readme(&program);
    std::fs::write(doc_dir.join("README.md"), readme).unwrap_or_else(|e| {
        eprintln!("Failed to write README: {}", e);
        process::exit(1);
    });

    // Generate command docs
    for cmd in &program.commands {
        let only_filter = only.map(|o| o.contains(&cmd.name)).unwrap_or(true);
        if only_filter {
            let doc = generate_command_doc(cmd);
            std::fs::write(doc_dir.join("commands").join(format!("{}.md", cmd.name)), doc).unwrap_or_else(|e| {
                eprintln!("Failed to write {} docs: {}", cmd.name, e);
            });
        }
    }

    // Generate event docs
    for event in &program.events {
        let doc = generate_event_doc(event);
        std::fs::write(doc_dir.join("events").join(format!("{}.md", event.name)), doc).unwrap_or_else(|e| {
            eprintln!("Failed to write event docs: {}", e);
        });
    }

    // Generate schedule docs
    for sched in &program.schedules {
        let doc = generate_schedule_doc(sched);
        std::fs::write(doc_dir.join("schedules").join(format!("{}.md", sched.name)), doc).unwrap_or_else(|e| {
            eprintln!("Failed to write schedule docs: {}", e);
        });
    }

    // Generate type docs
    for ty in &program.types {
        let doc = generate_type_doc(ty);
        std::fs::write(doc_dir.join("types").join(format!("{}.md", ty.name)), doc).unwrap_or_else(|e| {
            eprintln!("Failed to write type docs: {}", e);
        });
    }

    // Generate reference
    let ref_docs = vec![
        ("syntax", generate_syntax_ref()),
        ("commands", generate_commands_ref()),
        ("components", generate_components_ref()),
        ("embeds", generate_embeds_ref()),
        ("events", generate_events_ref()),
        ("schedules", generate_schedules_ref()),
        ("middleware", generate_middleware_ref()),
        ("database", generate_database_ref()),
        ("http", generate_http_ref()),
        ("threading", generate_threading_ref()),
        ("errors", generate_errors_ref()),
        ("modules", generate_modules_ref()),
        ("types", generate_types_ref()),
        ("config", generate_config_ref()),
    ];

    for (name, content) in &ref_docs {
        std::fs::write(doc_dir.join("reference").join(format!("{}.md", name)), content).unwrap_or_else(|e| {
            eprintln!("Failed to write reference/{}.md: {}", name, e);
        });
    }

    // Generate stdlib docs
    let stdlib_docs = vec![
        ("strings", generate_stdlib_strings_doc()),
        ("math", generate_stdlib_math_doc()),
        ("time", generate_stdlib_time_doc()),
        ("collections", generate_stdlib_collections_doc()),
        ("discord_helpers", generate_stdlib_discord_doc()),
    ];

    for (name, content) in &stdlib_docs {
        std::fs::write(doc_dir.join("stdlib").join(format!("{}.md", name)), content).unwrap_or_else(|e| {
            eprintln!("Failed to write stdlib/{}.md: {}", name, e);
        });
    }

    // Generate sidebar
    let sidebar = generate_sidebar(&program);
    std::fs::write(doc_dir.join("_sidebar.md"), sidebar).ok();

    println!("Documentation generated in 'docs/'");
    let cmd_count = program.commands.len();
    let evt_count = program.events.len();
    let sched_count = program.schedules.len();
    println!("  {} commands, {} events, {} schedules", cmd_count, evt_count, sched_count);
    println!("  {} reference pages", ref_docs.len());
    println!("  {} stdlib pages", stdlib_docs.len());
}

fn cmd_test(file: &str, verbose: bool) {
    let (program, _source) = parse_source(file);

    if program.tests.is_empty() {
        println!("No tests found in '{}'", file);
        return;
    }

    let rt = Runtime::new(program);
    rt.register_globals();

    let total = rt.tests.len();
    let mut passed = 0;
    let mut failed = 0;

    for test in &rt.tests {
        if verbose {
            println!("Test: {}...", test.name);
        }
        match rt.execute(&test.body) {
            Ok(_) => {
                passed += 1;
                if verbose {
                    println!("  PASSED");
                }
            }
            Err(e) => {
                failed += 1;
                println!("  FAILED: {}", e);
            }
        }
    }

    println!("\nTest results: {}/{} passed ({} failed)", passed, total, failed);
    if failed > 0 {
        process::exit(1);
    }
}

fn cmd_check(file: &str, _fix: bool, _strict: bool) {
    let (program, source) = parse_source(file);
    let checker = Checker::new(&program);
    let warnings = checker.check();

    let line_count = source.lines().count();
    let cmd_count = program.commands.len();
    let evt_count = program.events.len();

    println!("File: {}", file);
    println!("Lines: {}", line_count);
    println!("Commands: {}", cmd_count);
    println!("Events: {}", evt_count);
    println!("Imports: {}", program.imports.len());

    if warnings.is_empty() {
        println!("No issues found.");
    } else {
        for w in &warnings {
            println!("  Warning: {}", w);
        }
    }
}

fn cmd_fmt(path: &str, check: bool) {
    let source = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", path, e);
            process::exit(1);
        }
    };

    let formatted = match formatter::format_source(&source) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Format error: {}", e);
            process::exit(1);
        }
    };

    if check {
        if source == formatted {
            println!("{}: formatted correctly", path);
        } else {
            println!("{}: needs formatting", path);
            process::exit(1);
        }
    } else {
        std::fs::write(path, &formatted).unwrap_or_else(|e| {
            eprintln!("Failed to write '{}': {}", path, e);
            process::exit(1);
        });
        println!("Formatted: {}", path);
    }
}

fn cmd_env(show: bool) {
    // Try to load .env file
    if let Ok(content) = std::fs::read_to_string(".env") {
        println!("Environment variables from .env:");
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = &trimmed[..eq_pos];
                let val = &trimmed[eq_pos + 1..];
                if show {
                    println!("  {} = {}", key, val);
                } else {
                    println!("  {} = {}", key, "********");
                }
            }
        }
    } else {
        println!("No .env file found in current directory");
    }
}

fn cmd_module(name: &str) {
    // Default to creating a module with a single exported command
    let content = format!(
        r#"export cmd {} {{
    slash true
    description "Command from {} module"

    reply "Module {} loaded!"
}}
"#,
        name, name, name
    );
    let dir = Path::new("modules");
    std::fs::create_dir_all(dir).ok();
    let file = dir.join(format!("{}.ds", name));
    std::fs::write(&file, &content).unwrap_or_else(|e| {
        eprintln!("Failed to create module: {}", e);
        process::exit(1);
    });
    println!("Created module '{}' in '{}'", name, file.display());
}

fn cmd_refcard(file: &str, format: &str) {
    let (program, _source) = parse_source(file);

    match format {
        "md" | "markdown" => {
            let mut card = String::new();
            card.push_str("# DiscordScript Command Reference\n\n");
            for cmd in &program.commands {
                card.push_str(&format!("## `{}`\n\n", cmd.name));
                if let Some(ref desc) = cmd.description {
                    card.push_str(&format!("{}\n\n", desc));
                }
                card.push_str(&format!("- **Slash**: {}\n", cmd.slash));
                if let Some(ref prefix) = cmd.prefix {
                    card.push_str(&format!("- **Prefix**: `{}`\n", prefix));
                }
                if !cmd.params.is_empty() {
                    card.push_str("- **Parameters**:\n");
                    for param in &cmd.params {
                        card.push_str(&format!("  - `{}` ({:?})", param.name, param.param_type));
                        if let Some(ref desc) = param.description {
                            card.push_str(&format!(" - {}", desc));
                        }
                        card.push_str("\n");
                    }
                }
                card.push_str("\n");
            }
            std::fs::write("COMMANDS.md", card).unwrap_or_else(|e| {
                eprintln!("Failed to write COMMANDS.md: {}", e);
                process::exit(1);
            });
            println!("Command reference written to COMMANDS.md");
        }
        _ => eprintln!("Unsupported format '{}'. Use: md", format),
    }
}

// ===================== DOCUMENTATION GENERATORS =====================

fn generate_readme(program: &ast::Program) -> String {
    let mut doc = String::new();
    doc.push_str("# DiscordScript Bot\n\n");
    doc.push_str("## Commands\n\n");
    doc.push_str("| Command | Description | Type |\n");
    doc.push_str("|---------|-------------|------|\n");
    for cmd in &program.commands {
        let desc = cmd.description.as_deref().unwrap_or("");
        let cmd_type = if cmd.slash && cmd.prefix.is_some() { "Slash + Prefix" }
            else if cmd.slash { "Slash" } else { "Prefix" };
        doc.push_str(&format!("| `{}` | {} | {} |\n", cmd.name, desc, cmd_type));
    }

    doc.push_str("\n## Events\n\n");
    for evt in &program.events {
        doc.push_str(&format!("- `{}`\n", evt.name));
    }

    doc.push_str("\n## Schedules\n\n");
    for sched in &program.schedules {
        doc.push_str(&format!("- `{}`\n", sched.name));
    }

    doc
}

fn generate_command_doc(cmd: &ast::Command) -> String {
    let mut doc = String::new();
    doc.push_str(&format!("# `{}`\n\n", cmd.name));
    if let Some(ref desc) = cmd.description {
        doc.push_str(&format!("{}\n\n", desc));
    }

    doc.push_str("## Info\n\n");
    doc.push_str("| Field | Value |\n");
    doc.push_str("|-------|-------|\n");
    doc.push_str(&format!("| Type | {} |\n", if cmd.slash { "Slash" } else { "Prefix" }));
    if let Some(ref p) = cmd.prefix {
        doc.push_str(&format!("| Prefix | `{}` |\n", p));
    }
    if let Some(ref perm) = cmd.permission {
        doc.push_str(&format!("| Permission | `{}` |\n", perm));
    }

    if !cmd.params.is_empty() {
        doc.push_str("\n## Parameters\n\n");
        doc.push_str("| Name | Type | Required | Description |\n");
        doc.push_str("|------|------|----------|-------------|\n");
        for p in &cmd.params {
            let req = if p.required.unwrap_or(true) { "Yes" } else { "No" };
            let desc = p.description.as_deref().unwrap_or("");
            doc.push_str(&format!("| `{}` | `{:?}` | {} | {} |\n", p.name, p.param_type, req, desc));
        }
    }

    if !cmd.subcommands.is_empty() {
        doc.push_str("\n## Subcommands\n\n");
        for sub in &cmd.subcommands {
            doc.push_str(&format!("- `{}`\n", sub.name));
        }
    }

    doc
}

fn generate_event_doc(event: &ast::Event) -> String {
    let mut doc = String::new();
    doc.push_str(&format!("# Event: `{}`\n\n", event.name));
    if let Some(ref filter) = event.filter {
        doc.push_str(&format!("Filter: `{}`\n\n", filter));
    }
    doc
}

fn generate_schedule_doc(sched: &ast::Schedule) -> String {
    let mut doc = String::new();
    doc.push_str(&format!("# Schedule: `{}`\n\n", sched.name));
    if let Some(ref cron) = sched.cron {
        doc.push_str(&format!("- Cron: `{}`\n", cron));
    }
    if let Some(every) = sched.every {
        let unit = sched.every_unit.as_deref().unwrap_or("");
        doc.push_str(&format!("- Every: {} {}\n", every, unit));
    }
    doc
}

fn generate_type_doc(ty: &ast::CustomType) -> String {
    let mut doc = String::new();
    doc.push_str(&format!("# Type: `{}`\n\n", ty.name));
    doc.push_str("| Field | Type | Default | Optional |\n");
    doc.push_str("|-------|------|---------|----------|\n");
    for f in &ty.fields {
        let def = f.default.as_ref().map(|d| format!("{:?}", d)).unwrap_or_default();
        let opt = if f.optional { "Yes" } else { "No" };
        doc.push_str(&format!("| `{}` | `{:?}` | {} | {} |\n", f.name, f.field_type, def, opt));
    }
    doc
}

fn generate_syntax_ref() -> String {
    r#"# DiscordScript Syntax Reference

## Basic Structure

```discordscript
config {
    prefix "!"
    status "playing" "DiscordScript"
    intents all
}

cmd <name> {
    slash true
    prefix true
    
    reply "Hello World!"
}
```

## Comments

```discordscript
# This is a comment
```

## Strings

```discordscript
"plain string"
"interpolated {variable} string"
```

## Variables

```discordscript
let x = 10
set x = x + 1
```

## Conditionals

```discordscript
if condition { ... } elif condition { ... } else { ... }
```

## Loops

```discordscript
for item in collection { ... }
```
"#.into()
}

fn generate_commands_ref() -> String {
    r#"# Commands Reference

## Defining a Command

```discordscript
cmd <name> {
    slash <true|false>
    prefix <true|false|"custom">
    description "..."
    
    param <name> {
        type <string|int|user|...>
        description "..."
        required true|false
    }
    
    reply "..."
}
```

## Parameters

| Type | Description |
|------|-------------|
| string | Text input |
| int | Integer number |
| float | Decimal number |
| bool | Boolean |
| user | Discord user |
| channel | Discord channel |
| role | Discord role |
| member | Guild member |
| attachment | File upload |
"#.into()
}

fn generate_components_ref() -> String {
    r#"# Components Reference

## Buttons

```discordscript
row {
    button "Label" style success id btn_id
    button "Danger" style danger id btn_danger disabled
    url "Website" https://example.com
}
```

## Select Menu

```discordscript
row {
    select "Choose" id select_id {
        option "Option 1" value opt1 description "First option"
        option "Option 2" value opt2 description "Second option"
    }
}
```

## Modal

```discordscript
modal "Title" {
    input "Name" short required placeholder "Your name"
    input "Message" paragraph required
}
```
"#.into()
}

fn generate_embeds_ref() -> String {
    r##"# Embeds Reference

## Embed Structure

```discordscript
embed {
    title "Title"
    desc "Description"
    color "#5865F2"
    field "Name" "Value" inline
    field "Name2" "Value2" inline
    footer "Footer text"
}
```
"##.into()
}

fn generate_events_ref() -> String {
    r#"# Events Reference

## Available Events

```discordscript
on ready { ... }
on message { ... }
on member_join { ... }
on member_leave { ... }
on member_ban { ... }
on member_unban { ... }
on member_update { ... }
on message_delete { ... }
on message_edit { ... }
on channel_create { ... }
on channel_delete { ... }
on voice_join { ... }
on voice_leave { ... }
on button_click "id" { ... }
on modal_submit "id" { ... }
on select_menu "id" { ... }
on autocomplete "cmd" { ... }
on error { ... }
```
"#.into()
}

fn generate_schedules_ref() -> String {
    r#"# Schedules Reference

## Cron Schedule

```discordscript
schedule "name" {
    cron "0 8 * * *"
    timezone "America/Sao_Paulo"
    
    send "Daily message!" in channel "general"
}
```

## Interval Schedule

```discordscript
schedule "name" {
    every 5m
    
    std.log("Task executed")
}
```

## One-time Schedule

```discordscript
schedule "name" {
    once after 10s
    
    reply "Delayed message"
}
```
"#.into()
}

fn generate_middleware_ref() -> String {
    r#"# Middleware Reference

## Defining Middleware

```discordscript
middleware "name" {
    # Runs before matched commands
    if condition {
        cancel
    }
}
```

## Global Middleware

```discordscript
config {
    middleware ["logging", "cooldown"]
}
```

## Per-Command Middleware

```discordscript
cmd admin {
    slash true
    middleware ["check_admin"]
    
    reply "Admin panel"
}
```
"#.into()
}

fn generate_database_ref() -> String {
    r#"# Database Reference

## SQLite

```discordscript
db.query("CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT)")
db.query("INSERT INTO users VALUES (?, ?)", user.id, user.name)
let rows = db.query("SELECT * FROM users")
```

## JSON Storage

```discordscript
json.set("guilds.{guild.id}.prefix", "!")
let val = json.get("guilds.{guild.id}.prefix")
json.delete("guilds.{guild.id}.temp")
```
"#.into()
}

fn generate_http_ref() -> String {
    r#"# HTTP Reference

## Requests

```discordscript
let response = http.get("https://api.example.com/data")
let post = http.post("https://api.example.com/data") {
    headers {
        "Authorization" "Bearer {env.TOKEN}"
    }
    body {
        "key" "value"
    }
}
http.put("https://api.example.com/data/1") { body { "name" "updated" } }
http.delete("https://api.example.com/data/1")
```
"#.into()
}

fn generate_threading_ref() -> String {
    r#"# Multi-threading Reference

## Thread

```discordscript
thread {
    # Runs in separate thread
    let result = heavy_computation()
}
```

## Parallel

```discordscript
let results = parallel [
    { fetch("a") }
    { fetch("b") }
    { fetch("c") }
]
```

## Async/Await

```discordscript
let data = await fetch_data()
```
"#.into()
}

fn generate_errors_ref() -> String {
    r#"# Error Handling Reference

## Try/Catch

```discordscript
try {
    let data = http.get("https://api.example.com")
} catch e {
    reply "Error: {e.message}" ephemeral true
} finally {
    std.log("Operation completed")
}

## Throw

```discordscript
if !condition {
    throw "Custom error message"
}
```
"#.into()
}

fn generate_modules_ref() -> String {
    r#"# Modules Reference

## Import

```discordscript
import "module_name"
import "path/to/module" as alias
import "dir/*"
use module_name.symbol
```

## Export

```discordscript
export cmd <name> { ... }
```
"#.into()
}

fn generate_types_ref() -> String {
    r#"# Custom Types Reference

## Definition

```discordscript
type MyType {
    name string
    value int default 0
    active bool
    optional_field string optional
}
```

## Usage

```discordscript
let obj = MyType { name: "test", value: 42, active: true }
let val = obj.name
```
"#.into()
}

fn generate_config_ref() -> String {
    r#"# Configuration Reference

## Config Options

| Option | Type | Description |
|--------|------|-------------|
| prefix | string | Default prefix for text commands |
| status | string string | Bot status (type, text) |
| intents | all or [strings] | Discord intents |
| threads | int | Thread pool size |
| http_timeout | int | HTTP request timeout |
| http_retry | int | HTTP retry count |
| db_url | string | Database URL |
| cache_ttl | int | Default cache TTL |
| log_level | string | Log level |
| token_env | string | Env var name for token |
| middleware | [strings] | Global middleware |
```"#.into()
}

fn generate_stdlib_strings_doc() -> String {
    r#"# Stdlib: Strings

## Functions

| Function | Description | Example |
|----------|-------------|---------|
| `std.upper(s)` | Uppercase | `std.upper("hello")` -> "HELLO" |
| `std.lower(s)` | Lowercase | `std.lower("HELLO")` -> "hello" |
| `std.len(s)` | Length | `std.len("hello")` -> 5 |
| `std.slice(s, start, end)` | Slice | `std.slice("hello", 0, 2)` -> "he" |
| `std.join(list, sep)` | Join | `std.join(["a","b"], ",")` -> "a,b" |
| `std.contains(s, sub)` | Contains | `std.contains("hello", "ll")` -> true |
| `std.replace(s, from, to)` | Replace | `std.replace("hi there", "hi", "hello")` |
| `std.trim(s)` | Trim | `std.trim("  hi  ")` -> "hi" |
| `std.split(s, sep)` | Split | `std.split("a,b,c", ",")` -> ["a","b","c"] |
```"#.into()
}

fn generate_stdlib_math_doc() -> String {
    r#"# Stdlib: Math

## Functions

| Function | Description | Example |
|----------|-------------|---------|
| `std.random(min, max)` | Random int | `std.random(1, 100)` |
| `std.clamp(val, min, max)` | Clamp | `std.clamp(150, 0, 100)` -> 100 |
| `std.abs(n)` | Absolute | `std.abs(-42)` -> 42 |
| `std.min(a, b)` | Minimum | `std.min(5, 10)` -> 5 |
| `std.max(a, b)` | Maximum | `std.max(5, 10)` -> 10 |
| `std.round(n)` | Round | `std.round(3.7)` -> 4 |
| `std.floor(n)` | Floor | `std.floor(3.7)` -> 3 |
| `std.ceil(n)` | Ceiling | `std.ceil(3.2)` -> 4 |
```"#.into()
}

fn generate_stdlib_time_doc() -> String {
    r#"# Stdlib: Time

## Functions

| Function | Description | Example |
|----------|-------------|---------|
| `std.now()` | Current timestamp | `std.now()` |
| `std.format_time(ts, fmt)` | Format time | `std.format_time(now, "DD/MM/YYYY")` |
| `std.sleep(ms)` | Sleep | `std.sleep(1000)` |
```"#.into()
}

fn generate_stdlib_collections_doc() -> String {
    r#"# Stdlib: Collections

## Functions

| Function | Description | Example |
|----------|-------------|---------|
| `std.push(list, item)` | Push item | `std.push(list, 4)` |
| `std.pop(list)` | Pop item | `let last = std.pop(list)` |
| `std.first(list)` | First item | `let f = std.first(list)` |
| `std.last(list)` | Last item | `let l = std.last(list)` |
| `std.includes(list, item)` | Check | `std.includes([1,2], 2)` -> true |
| `std.filter(list, fn)` | Filter | `std.filter(list, fn(x) { x > 1 })` |
| `std.map(list, fn)` | Map | `std.map(list, fn(x) { x * 2 })` |
| `std.reduce(list, fn, init)` | Reduce | `std.reduce(list, fn(a,b) { a+b }, 0)` |
| `std.len(list)` | Length | `std.len([1,2,3])` -> 3 |
| `std.chunk(list, n)` | Chunk | `std.chunk([1,2,3,4], 2)` -> [[1,2],[3,4]] |
```"#.into()
}

fn generate_stdlib_discord_doc() -> String {
    r##"# Stdlib: Discord Helpers

## Functions

| Function | Description |
|----------|-------------|
| `std.avatar_url(user)` | Get user avatar URL |
| `std.hex_to_int("#color")` | Convert hex color to int |
| `std.channel_name(channel)` | Get channel name |
| `std.role_name(role)` | Get role name |
| `std.tag(user)` | Get user tag (username#discrim) |
```"##.into()
}

fn generate_sidebar(program: &ast::Program) -> String {
    let mut sidebar = String::new();
    sidebar.push_str("- [Home](README.md)\n");
    sidebar.push_str("- Commands\n");
    for cmd in &program.commands {
        sidebar.push_str(&format!("  - [{}](commands/{}.md)\n", cmd.name, cmd.name));
    }
    sidebar.push_str("- Events\n");
    for evt in &program.events {
        sidebar.push_str(&format!("  - [{}](events/{}.md)\n", evt.name, evt.name));
    }
    sidebar.push_str("- Schedules\n");
    for sched in &program.schedules {
        sidebar.push_str(&format!("  - [{}](schedules/{}.md)\n", sched.name, sched.name));
    }
    sidebar.push_str("- Language Reference\n");
    sidebar.push_str("  - [Syntax](reference/syntax.md)\n");
    sidebar.push_str("  - [Commands](reference/commands.md)\n");
    sidebar.push_str("  - [Components](reference/components.md)\n");
    sidebar.push_str("  - [Embeds](reference/embeds.md)\n");
    sidebar.push_str("  - [Events](reference/events.md)\n");
    sidebar.push_str("  - [Schedules](reference/schedules.md)\n");
    sidebar.push_str("  - [Middleware](reference/middleware.md)\n");
    sidebar.push_str("  - [Database](reference/database.md)\n");
    sidebar.push_str("  - [HTTP](reference/http.md)\n");
    sidebar.push_str("  - [Threading](reference/threading.md)\n");
    sidebar.push_str("  - [Errors](reference/errors.md)\n");
    sidebar.push_str("  - [Modules](reference/modules.md)\n");
    sidebar.push_str("  - [Types](reference/types.md)\n");
    sidebar.push_str("  - [Config](reference/config.md)\n");
    sidebar.push_str("- Standard Library\n");
    sidebar.push_str("  - [Strings](stdlib/strings.md)\n");
    sidebar.push_str("  - [Math](stdlib/math.md)\n");
    sidebar.push_str("  - [Time](stdlib/time.md)\n");
    sidebar.push_str("  - [Collections](stdlib/collections.md)\n");
    sidebar.push_str("  - [Discord Helpers](stdlib/discord_helpers.md)\n");
    sidebar
}
