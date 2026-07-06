use crate::ast::*;
use std::io::Write;

fn ensure_dir(path: &str) {
    std::fs::create_dir_all(path).unwrap_or_else(|e| {
        eprintln!("Failed to create directory '{}': {}", path, e);
    });
}

fn write_file(path: &str, content: &str) {
    std::fs::write(path, content).unwrap_or_else(|e| {
        eprintln!("Failed to write '{}': {}", path, e);
    });
}

fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Literal(lit) => match lit {
            Literal::String(s, _) => s.clone(),
            Literal::Integer(n) => n.to_string(),
            Literal::Float(f) => f.to_string(),
            Literal::Boolean(b) => b.to_string(),
            Literal::Null => "None".into(),
        },
        Pattern::Variable(name) => name.clone(),
        Pattern::MemberAccess(obj, field) => format!("{}.{}", pattern_to_string(obj), field),
        Pattern::Call(func, args) => {
            let func_str = pattern_to_string(func);
            let args_str: Vec<String> = args.iter().map(pattern_to_string).collect();
            format!("{}({})", func_str, args_str.join(", "))
        }
        Pattern::BinOp(left, op, right) => {
            let op_str = match op {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => " * ",
                BinOp::Div => " / ",
                BinOp::Mod => " % ",
                BinOp::Eq => " == ",
                BinOp::Neq => " != ",
                BinOp::Lt => " < ",
                BinOp::Gt => " > ",
                BinOp::Lte => " <= ",
                BinOp::Gte => " >= ",
                BinOp::And => " and ",
                BinOp::Or => " or ",
                BinOp::Assign => " = ",
            };
            format!("{}{}{}", pattern_to_string(left), op_str, pattern_to_string(right))
        }
        Pattern::List(items) => {
            let items_str: Vec<String> = items.iter().map(pattern_to_string).collect();
            format!("[{}]", items_str.join(", "))
        }
        Pattern::Object(fields) => {
            let fields_str: Vec<String> = fields.iter()
                .map(|(k, v)| format!("\"{}\": {}", k, pattern_to_string(v)))
                .collect();
            format!("{{{}}}", fields_str.join(", "))
        }
        Pattern::Interpolated(parts) => {
            let mut result = String::from("f\"");
            for part in parts {
                match part {
                    Pattern::Literal(Literal::String(s, _)) => result.push_str(s),
                    _ => result.push_str(&format!("{{{}}}", pattern_to_string(part))),
                }
            }
            result.push('"');
            result
        }
        Pattern::Index(obj, index) => {
            format!("{}[{}]", pattern_to_string(obj), pattern_to_string(index))
        }
        Pattern::EnumVariant(enum_name, variant_name, _fields) => {
            format!("{}.{}", enum_name, variant_name)
        }
    }
}

fn translate_body(statements: &[Statement], f: &dyn Fn(&Statement, usize, &str) -> String, indent: usize, cmd: &str) -> String {
    let mut result = String::new();
    for stmt in statements {
        result.push_str(&f(stmt, indent, cmd));
    }
    result
}

fn translate_set(value: &Pattern, target: &Pattern, prefix: &str) -> String {
    match target {
        Pattern::Variable(name) => format!("{}{} = {}\n", prefix, name, pattern_to_string(value)),
        Pattern::MemberAccess(obj, field) => format!("{}{}.{} = {}\n", prefix, pattern_to_string(obj), field, pattern_to_string(value)),
        Pattern::Index(obj, index) => format!("{}{}[{}] = {}\n", prefix, pattern_to_string(obj), pattern_to_string(index), pattern_to_string(value)),
        _ => format!("{}pass  # set target\n", prefix),
    }
}

fn translate_embed(embed: &Embed, prefix: &str) -> String {
    let mut result = format!("{}embed = discord.Embed(\n", prefix);
    if let Some(ref title) = embed.title {
        result.push_str(&format!("{}    title=\"{}\",\n", prefix, pattern_to_string(title)));
    }
    if let Some(ref desc) = embed.description {
        result.push_str(&format!("{}    description=\"{}\",\n", prefix, pattern_to_string(desc)));
    }
    if let Some(ref color) = embed.color {
        result.push_str(&format!("{}    color=int(\"{}\", 16),\n", prefix, pattern_to_string(color)));
    }
    result.push_str(&format!("{})\n", prefix));
    for field in &embed.fields {
        result.push_str(&format!("{}embed.add_field(name=\"{}\", value=\"{}\", inline={})\n",
            prefix, field.name, pattern_to_string(&field.value), field.inline));
    }
    if let Some(ref footer) = embed.footer {
        result.push_str(&format!("{}embed.set_footer(text=\"{}\")\n", prefix, pattern_to_string(footer)));
    }
    if let Some(ref image) = embed.image {
        result.push_str(&format!("{}embed.set_image(url=\"{}\")\n", prefix, image));
    }
    if let Some(ref thumb) = embed.thumbnail {
        result.push_str(&format!("{}embed.set_thumbnail(url=\"{}\")\n", prefix, thumb));
    }
    result
}

fn translate_if(condition: &Pattern, body: &[Statement], elifs: &[(Pattern, Vec<Statement>)], else_body: &Option<Vec<Statement>>,
                f: &dyn Fn(&Statement, usize, &str) -> String, indent: usize, cmd: &str) -> String {
    let prefix = "    ".repeat(indent);
    let mut result = format!("{}if {}:\n", prefix, pattern_to_string(condition));
    result.push_str(&translate_body(body, f, indent + 1, cmd));
    for (elif_cond, elif_body) in elifs {
        result.push_str(&format!("{}elif {}:\n", prefix, pattern_to_string(elif_cond)));
        result.push_str(&translate_body(elif_body, f, indent + 1, cmd));
    }
    if let Some(eb) = else_body {
        result.push_str(&format!("{}else:\n", prefix));
        result.push_str(&translate_body(eb, f, indent + 1, cmd));
    }
    result
}

fn translate_for(var: &str, iterable: &Pattern, body: &[Statement],
                 f: &dyn Fn(&Statement, usize, &str) -> String, indent: usize, cmd: &str) -> String {
    let prefix = "    ".repeat(indent);
    let mut result = format!("{}for {} in {}:\n", prefix, var, pattern_to_string(iterable));
    result.push_str(&translate_body(body, f, indent + 1, cmd));
    result
}

fn translate_try_catch(try_body: &[Statement], catch_var: &Option<String>, catch_body: &[Statement], finally_body: &Option<Vec<Statement>>,
                       f: &dyn Fn(&Statement, usize, &str) -> String, indent: usize, cmd: &str) -> String {
    let prefix = "    ".repeat(indent);
    let mut result = format!("{}try:\n", prefix);
    result.push_str(&translate_body(try_body, f, indent + 1, cmd));
    if !catch_body.is_empty() {
        if let Some(var) = catch_var {
            result.push_str(&format!("{}except Exception as {}:\n", prefix, var));
        } else {
            result.push_str(&format!("{}except Exception:\n", prefix));
        }
        result.push_str(&translate_body(catch_body, f, indent + 1, cmd));
    }
    if let Some(fb) = finally_body {
        result.push_str(&format!("{}finally:\n", prefix));
        result.push_str(&translate_body(fb, f, indent + 1, cmd));
    }
    result
}

fn translate_throw(message: &Pattern, prefix: &str) -> String {
    format!("{}raise Exception(\"{}\")\n", prefix, pattern_to_string(message))
}

fn translate_return(value: &Option<Pattern>, prefix: &str) -> String {
    match value {
        Some(v) => format!("{}return {}\n", prefix, pattern_to_string(v)),
        None => format!("{}return\n", prefix),
    }
}

fn translate_log(message: &Pattern, level: &str, prefix: &str) -> String {
    let py_level = match level {
        "error" => "error",
        "warn" => "warning",
        "debug" => "debug",
        _ => "info",
    };
    format!("{}import logging\n{}logging.logger.{}(\"{}\")\n", prefix, prefix, py_level, pattern_to_string(message))
}

fn translate_db(query: &str, params: &[Pattern], prefix: &str) -> String {
    if params.is_empty() {
        format!("{}cursor.execute(\"{}\")\n{}results = cursor.fetchall()\n", prefix, query, prefix)
    } else {
        let params_str: Vec<String> = params.iter().map(pattern_to_string).collect();
        format!("{}cursor.execute(\"{}\", ({}))\n{}results = cursor.fetchall()\n", prefix, query, params_str.join(", "), prefix)
    }
}

fn translate_discord(action: &str, args: &[Pattern], prefix: &str) -> String {
    let args_str: Vec<String> = args.iter().map(pattern_to_string).collect();
    format!("{}await ctx.invoke(self.bot.get_command(\"{}\"), {})\n", prefix, action, args_str.join(", "))
}

fn translate_json(op: &str, key: &Pattern, value: &Option<Pattern>, prefix: &str) -> String {
    let key_str = pattern_to_string(key);
    match op {
        "set" | "push" => {
            if let Some(v) = value {
                format!("{}data[\"{}\"] = {}\n", prefix, key_str, pattern_to_string(v))
            } else {
                String::new()
            }
        }
        "get" => format!("{}data.get(\"{}\", None)\n", prefix, key_str),
        "delete" => format!("{}data.pop(\"{}\", None)\n", prefix, key_str),
        _ => String::new(),
    }
}

fn translate_send(content: &Pattern, channel: &Option<Pattern>, prefix: &str) -> String {
    if let Some(ch) = channel {
        let ch_str = pattern_to_string(ch);
        format!("{}channel = discord.utils.get(ctx.guild.channels, name=\"{}\")\n{}if channel:\n{}    await channel.send(\"{}\")\n",
            prefix, ch_str, prefix, prefix, pattern_to_string(content))
    } else {
        format!("{}await ctx.send(\"{}\")\n", prefix, pattern_to_string(content))
    }
}

fn translate_reply(content: &Pattern, ephemeral: &Option<bool>, prefix: &str, backend: &str) -> String {
    let content_str = pattern_to_string(content);
    match backend {
        "nextcord" => {
            if *ephemeral == Some(true) {
                format!("{}await interaction.response.send_message(\"{}\", ephemeral=True)\n", prefix, content_str)
            } else {
                format!("{}await interaction.response.send_message(\"{}\")\n", prefix, content_str)
            }
        }
        "pycord" => {
            if *ephemeral == Some(true) {
                format!("{}await ctx.respond(\"{}\", ephemeral=True)\n", prefix, content_str)
            } else {
                format!("{}await ctx.respond(\"{}\")\n", prefix, content_str)
            }
        }
        _ => {
            if *ephemeral == Some(true) {
                format!("{}await ctx.respond(\"{}\", ephemeral=True)\n", prefix, content_str)
            } else {
                format!("{}await ctx.send(\"{}\")\n", prefix, content_str)
            }
        }
    }
}

fn translate_std_call(module: &str, func: &str, args: &[Pattern], prefix: &str) -> String {
    let args_str: Vec<String> = args.iter().map(pattern_to_string).collect();
    format!("{}{}_{}({})\n", prefix, module, func, args_str.join(", "))
}

fn translate_paginate(items: &Pattern, per_page: i64, prefix: &str) -> String {
    let items_str = pattern_to_string(items);
    format!("{}for i in range(0, len({}), {}):\n{}    page = {}[i:i+{}]\n", prefix, items_str, per_page, prefix, items_str, per_page)
}

fn translate_thread(body: &[Statement], f: &dyn Fn(&Statement, usize, &str) -> String, indent: usize, cmd: &str) -> String {
    let prefix = "    ".repeat(indent);
    let mut result = format!("{}import threading\n{}def thread_task():\n", prefix, prefix);
    result.push_str(&translate_body(body, f, indent + 1, cmd));
    result.push_str(&format!("{}threading.Thread(target=thread_task).start()\n", prefix));
    result
}

fn common_translate(stmt: &Statement, indent: usize, cmd: &str, backend: &str) -> String {
    let prefix = "    ".repeat(indent);
    match stmt {
        Statement::Reply { content, ephemeral, .. } => translate_reply(content, ephemeral, &prefix, backend),
        Statement::Send { content, channel } => translate_send(content, channel, &prefix),
        Statement::Let { name, value } => format!("{}{} = {}\n", prefix, name, pattern_to_string(value)),
        Statement::Set { target, value } => translate_set(value, target, &prefix),
        Statement::Log { level, message, .. } => translate_log(message, level, &prefix),
        Statement::Embed(embed) => translate_embed(embed, &prefix),
        Statement::Expression(expr) => format!("{}{}\n", prefix, pattern_to_string(expr)),
        Statement::Return { value } => translate_return(value, &prefix),
        Statement::Throw { message } => translate_throw(message, &prefix),
        Statement::If { condition, body, elifs, else_body } => translate_if(condition, body, elifs, else_body, &|s, i, c| common_translate(s, i, c, backend), indent, cmd),
        Statement::For { var, iterable, body } => translate_for(var, iterable, body, &|s, i, c| common_translate(s, i, c, backend), indent, cmd),
        Statement::TryCatch { try_body, catch_var, catch_body, finally_body } => translate_try_catch(try_body, catch_var, catch_body, finally_body, &|s, i, c| common_translate(s, i, c, backend), indent, cmd),
        Statement::Thread { body } => translate_thread(body, &|s, i, c| common_translate(s, i, c, backend), indent, cmd),
        Statement::Parallel { branches, .. } => {
            let mut result = String::new();
            result.push_str(&format!("{}import threading\n", prefix));
            for (i, branch) in branches.iter().enumerate() {
                result.push_str(&format!("{}def branch_{}():\n", prefix, i));
                result.push_str(&translate_body(branch, &|s, i, c| common_translate(s, i, c, backend), indent + 1, cmd));
                result.push_str(&format!("{}threading.Thread(target=branch_{}).start()\n", prefix, i));
            }
            result
        }
        Statement::Await { expr, .. } => format!("{}await {}\n", prefix, pattern_to_string(expr)),
        Statement::DBQuery { query, params, .. } => translate_db(query, params, &prefix),
        Statement::DiscordAction { action, args } => translate_discord(action, args, &prefix),
        Statement::JSONOp { op, key, value, .. } => translate_json(op, key, value, &prefix),
        Statement::Paginate { items, per_page, .. } => translate_paginate(items, *per_page, &prefix),
        Statement::StdCall { module, func, args, .. } => translate_std_call(module, func, args, &prefix),
        Statement::Cancel => format!("{}return\n", prefix),
        Statement::CancelEvent => format!("{}return\n", prefix),
        Statement::Row(_) => format!("{}pass  # row components\n", prefix),
        Statement::Modal(_) => format!("{}pass  # modal\n", prefix),
        Statement::HTTPRequest { .. } => format!("{}pass  # http request\n", prefix),
        Statement::FileOp { .. } => format!("{}pass  # file op\n", prefix),
        Statement::VoiceOp { .. } => format!("{}pass  # voice op\n", prefix),
        Statement::WebhookOp { .. } => format!("{}pass  # webhook\n", prefix),
        Statement::CacheOp { .. } => format!("{}pass  # cache op\n", prefix),
        Statement::Autocomplete { .. } => format!("{}pass  # autocomplete\n", prefix),
        Statement::CustomEvent { .. } => format!("{}pass  # custom event\n", prefix),
        Statement::Mock { .. } => format!("{}pass  # mock\n", prefix),
        Statement::Assert { condition, message } => {
            let msg = message.as_deref().unwrap_or("assertion failed");
            format!("{}assert {}, \"{}\"\n", prefix, pattern_to_string(condition), msg)
        }
        Statement::Execute { .. } => format!("{}pass  # execute\n", prefix),
        Statement::FnCall { name, args, .. } => {
            let args_str: Vec<String> = args.iter().map(pattern_to_string).collect();
            format!("{}{}({})\n", prefix, name, args_str.join(", "))
        }
        Statement::While { condition, body } => {
            let cond_str = pattern_to_string(condition);
            let mut result = format!("{}while {}:\n", prefix, cond_str);
            for s in body {
                result.push_str(&common_translate(s, indent + 1, cmd, backend));
            }
            result
        }
        Statement::Break => format!("{}break\n", prefix),
        Statement::Continue => format!("{}continue\n", prefix),
        Statement::Match { value, arms, else_body } => {
            let val_str = pattern_to_string(value);
            let mut result = String::new();
            for (i, (pat, arm_body)) in arms.iter().enumerate() {
                let pat_str = pattern_to_string(pat);
                let kw = if i == 0 { "if" } else { "elif" };
                result.push_str(&format!("{}{} {} == {}:\n", prefix, kw, val_str, pat_str));
                for s in arm_body {
                    result.push_str(&common_translate(s, indent + 1, cmd, backend));
                }
            }
            if let Some(eb) = else_body {
                result.push_str(&format!("{}else:\n", prefix));
                for s in eb {
                    result.push_str(&common_translate(s, indent + 1, cmd, backend));
                }
            }
            result
        }
        Statement::TryExpr { body, catch_var, catch_body } => {
            let catch_var_name = catch_var.as_deref().unwrap_or("_err");
            let mut result = format!("{}try:\n", prefix);
            result.push_str(&common_translate(body, indent + 1, cmd, backend));
            result.push_str(&format!("{}except Exception as {}:\n", prefix, catch_var_name));
            result.push_str(&common_translate(catch_body, indent + 1, cmd, backend));
            result
        }
    }
}

// ===================== DISCORD.PY BACKEND =====================

pub fn generate_discordpy(program: &Program, out_dir: &str) {
    ensure_dir(out_dir);
    ensure_dir(&format!("{}/cogs", out_dir));

    let mut main = String::new();
    main.push_str("import discord\nfrom discord.ext import commands\nimport os\nimport json\nimport sqlite3\nimport logging\n\n");
    main.push_str("intents = discord.Intents.all()\n");
    main.push_str(&format!("bot = commands.Bot(command_prefix='{}', intents=intents)\n\n",
        program.config.as_ref().and_then(|c| c.prefix.as_deref()).unwrap_or("!")));

    main.push_str("bot.load_extension('cogs.commands')\n");
    main.push_str("bot.load_extension('cogs.events')\n\n");

    main.push_str("@bot.event\nasync def on_ready():\n");
    main.push_str("    await bot.change_presence(activity=discord.Game(name=\"DiscordScript\"))\n");
    main.push_str("    print(f\"Bot logged in as {bot.user}\")\n\n");

    main.push_str("if __name__ == '__main__':\n");
    main.push_str("    token = os.getenv('DISCORD_TOKEN', '')\n");
    main.push_str("    if not token:\n");
    main.push_str("        try:\n            from dotenv import load_dotenv\n            load_dotenv()\n            token = os.getenv('DISCORD_TOKEN', '')\n        except ImportError:\n            pass\n");
    main.push_str("    bot.run(token)\n");

    write_file(&format!("{}/main.py", out_dir), &main);

    let mut commands_py = String::new();
    commands_py.push_str("import discord\nfrom discord.ext import commands\nimport sqlite3\nimport json\nimport logging\n\n");
    commands_py.push_str("class Commands(commands.Cog):\n    def __init__(self, bot):\n        self.bot = bot\n\n");

    for cmd in &program.commands {
        commands_py.push_str(&format!("    @commands.command(name='{}')\n", cmd.name));
        commands_py.push_str(&format!("    async def {}(self, ctx", cmd.name));
        for param in &cmd.params {
            commands_py.push_str(&format!(", {}", param.name));
        }
        commands_py.push_str("):\n");
        if cmd.slash {
            commands_py.push_str(&format!("        \"\"\"{}\"\"\"\n", cmd.description.as_deref().unwrap_or("")));
        }
        for stmt in &cmd.body {
            commands_py.push_str(&common_translate(stmt, 2, &cmd.name, "discordpy"));
        }
        if cmd.body.is_empty() {
            commands_py.push_str("        pass\n");
        }
        commands_py.push_str("\n");
    }

    commands_py.push_str("async def setup(bot):\n    await bot.add_cog(Commands(bot))\n");
    write_file(&format!("{}/cogs/commands.py", out_dir), &commands_py);

    let mut events_py = String::new();
    events_py.push_str("import discord\nfrom discord.ext import commands\n\n");
    events_py.push_str("class Events(commands.Cog):\n    def __init__(self, bot):\n        self.bot = bot\n\n");

    for evt in &program.events {
        events_py.push_str("    @commands.Cog.listener()\n");
        let listener = match evt.name.as_str() {
            "ready" => "on_ready",
            "message" => "on_message",
            "member_join" => "on_member_join",
            "member_leave" => "on_member_remove",
            _ => &format!("on_{}", evt.name),
        };
        events_py.push_str(&format!("    async def {}(self):\n", listener));
        for stmt in &evt.body {
            events_py.push_str(&common_translate(stmt, 2, "", "discordpy"));
        }
        events_py.push_str("\n");
    }

    events_py.push_str("async def setup(bot):\n    await bot.add_cog(Events(bot))\n");
    write_file(&format!("{}/cogs/events.py", out_dir), &events_py);

    write_file(&format!("{}/requirements.txt", out_dir), "discord.py>=2.3\npython-dotenv\n");
    println!("  -> discord.py backend generated in '{}'", out_dir);
}

// ===================== NEXTCORD BACKEND =====================

pub fn generate_nextcord(program: &Program, out_dir: &str) {
    ensure_dir(out_dir);
    ensure_dir(&format!("{}/cogs", out_dir));

    let mut main = String::new();
    main.push_str("import nextcord\nfrom nextcord.ext import commands\nimport os\nimport json\nimport sqlite3\nimport logging\n\n");
    main.push_str("intents = nextcord.Intents.all()\n");
    main.push_str(&format!("bot = commands.Bot(command_prefix='{}', intents=intents)\n\n",
        program.config.as_ref().and_then(|c| c.prefix.as_deref()).unwrap_or("!")));

    for cmd in &program.commands {
        if cmd.slash {
            let params: Vec<String> = cmd.params.iter().map(|p| {
                let ptype = match p.param_type {
                    TypeName::String => "str",
                    TypeName::Int => "int",
                    TypeName::Bool => "bool",
                    TypeName::User => "nextcord.Member",
                    TypeName::Channel => "nextcord.TextChannel",
                    TypeName::Role => "nextcord.Role",
                    _ => "str",
                };
                format!("{}: {}", p.name, ptype)
            }).collect();

            main.push_str(&format!("@bot.slash_command(name=\"{}\", description=\"{}\")\n",
                cmd.name, cmd.description.as_deref().unwrap_or("")));
            main.push_str(&format!("async def {}(interaction: nextcord.Interaction", cmd.name));
            for p in &params { main.push_str(&format!(", {}", p)); }
            main.push_str("):\n");
            for stmt in &cmd.body {
                main.push_str(&common_translate(stmt, 1, &cmd.name, "nextcord"));
            }
            if cmd.body.is_empty() { main.push_str("    pass\n"); }
            main.push_str("\n");
        }
    }

    main.push_str("@bot.event\nasync def on_ready():\n    print(f'Bot logged in as {bot.user}')\n\n");
    main.push_str("if __name__ == '__main__':\n    token = os.getenv('DISCORD_TOKEN', '')\n");
    main.push_str("    if not token:\n        try:\n            from dotenv import load_dotenv\n            load_dotenv()\n            token = os.getenv('DISCORD_TOKEN', '')\n        except ImportError:\n            pass\n");
    main.push_str("    bot.run(token)\n");

    write_file(&format!("{}/main.py", out_dir), &main);
    write_file(&format!("{}/requirements.txt", out_dir), "nextcord>=2.6\npython-dotenv\n");
    println!("  -> nextcord backend generated in '{}'", out_dir);
}

// ===================== PY-CORD BACKEND =====================

pub fn generate_pycord(program: &Program, out_dir: &str) {
    ensure_dir(out_dir);

    let mut main = String::new();
    main.push_str("import discord\nfrom discord.ext import commands\nimport os\nimport json\nimport sqlite3\nimport logging\n\n");
    main.push_str("bot = commands.Bot(command_prefix='");
    main.push_str(program.config.as_ref().and_then(|c| c.prefix.as_deref()).unwrap_or("!"));
    main.push_str("', intents=discord.Intents.all())\n\n");

    for cmd in &program.commands {
        let params: Vec<String> = cmd.params.iter().map(|p| {
            let ptype = match p.param_type {
                TypeName::String => "str",
                TypeName::Int => "int",
                TypeName::Bool => "bool",
                TypeName::User => "discord.Member",
                TypeName::Channel => "discord.TextChannel",
                TypeName::Role => "discord.Role",
                _ => "str",
            };
            format!("{}: {}", p.name, ptype)
        }).collect();

        main.push_str(&format!("@bot.slash_command(name=\"{}\", description=\"{}\")\n",
            cmd.name, cmd.description.as_deref().unwrap_or("")));
        main.push_str(&format!("async def {}(ctx: discord.ApplicationContext", cmd.name));
        for p in &params { main.push_str(&format!(", {}", p)); }
        main.push_str("):\n");
        for stmt in &cmd.body {
            main.push_str(&common_translate(stmt, 1, &cmd.name, "pycord"));
        }
        if cmd.body.is_empty() { main.push_str("    pass\n"); }
        main.push_str("\n");
    }

    main.push_str("@bot.event\nasync def on_ready():\n    print(f'Bot logged in as {bot.user}')\n\n");
    main.push_str("if __name__ == '__main__':\n    bot.run(os.getenv('DISCORD_TOKEN'))\n");

    write_file(&format!("{}/main.py", out_dir), &main);
    write_file(&format!("{}/requirements.txt", out_dir), "py-cord>=2.6\npython-dotenv\n");
    println!("  -> py-cord backend generated in '{}'", out_dir);
}

// ===================== DISCLOUD BACKEND =====================

pub fn generate_discloud(program: &Program, out_dir: &str) {
    ensure_dir(out_dir);

    let mut main = String::new();
    main.push_str("import discord\nfrom discord.ext import commands\nimport os\nimport json\nimport sqlite3\nimport logging\n\n");
    main.push_str("intents = discord.Intents.all()\n");
    main.push_str(&format!("bot = commands.Bot(command_prefix='{}', intents=intents, help_command=None)\n\n",
        program.config.as_ref().and_then(|c| c.prefix.as_deref()).unwrap_or("!")));

    for cmd in &program.commands {
        let params: Vec<String> = cmd.params.iter().map(|p| {
            let ptype = match p.param_type {
                TypeName::String => "str",
                TypeName::Int => "int",
                TypeName::Bool => "bool",
                TypeName::User => "discord.Member",
                TypeName::Channel => "discord.TextChannel",
                TypeName::Role => "discord.Role",
                _ => "str",
            };
            format!("{}: {}", p.name, ptype)
        }).collect();

        main.push_str(&format!("@bot.command(name='{}')\n", cmd.name));
        main.push_str(&format!("async def {}(ctx: commands.Context", cmd.name));
        for p in &params { main.push_str(&format!(", {}", p)); }
        main.push_str("):\n");
        for stmt in &cmd.body {
            main.push_str(&common_translate(stmt, 1, &cmd.name, "discloud"));
        }
        if cmd.body.is_empty() { main.push_str("    pass\n"); }
        main.push_str("\n");
    }

    main.push_str("@bot.event\nasync def on_ready():\n    print(f'Bot logged in as {bot.user}')\n\n");
    main.push_str("if __name__ == '__main__':\n    bot.run(os.getenv('DISCORD_TOKEN'))\n");

    write_file(&format!("{}/main.py", out_dir), &main);

    let config_content = r#"{
    "name": "DiscordScript Bot",
    "language": "python",
    "main": "main.py",
    "autorestart": true,
    "ram": 512
}
"#;
    write_file(&format!("{}/discloud.config", out_dir), config_content);

    let runtime_content = r#"{"python": "3.11"}"#;
    write_file(&format!("{}/runtime.json", out_dir), runtime_content);

    write_file(&format!("{}/requirements.txt", out_dir), "discord.py>=2.3\n");

    let zip_path = format!("{}/../discloud_bot.zip", out_dir);
    if let Ok(file) = std::fs::File::create(&zip_path) {
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let files_to_zip = ["main.py", "discloud.config", "runtime.json", "requirements.txt"];
        for fname in &files_to_zip {
            let fpath = format!("{}/{}", out_dir, fname);
            if let Ok(content) = std::fs::read(&fpath) {
                zip.start_file(*fname, options).ok();
                zip.write_all(&content).ok();
            }
        }
        zip.finish().ok();
        println!("  -> Created discloud.zip at '{}'", zip_path);
    }

    println!("  -> discloud backend generated in '{}'", out_dir);
}
