use crate::ast::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    Function(String, Vec<String>),
    DiscordUser(String, String),
    DiscordChannel(String, String),
    DiscordRole(String, String),
    DiscordMember(String, String, Vec<String>),
    EmbedData(Embed),
}

pub struct Environment {
    pub variables: Arc<Mutex<HashMap<String, Value>>>,
    pub config: Option<Config>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: Arc::new(Mutex::new(HashMap::new())),
            config: None,
        }
    }

    pub fn set(&self, name: &str, value: Value) {
        let mut vars = self.variables.lock().unwrap();
        vars.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let vars = self.variables.lock().unwrap();
        vars.get(name).cloned()
    }

    pub fn get_mut(&self, name: &str) -> Option<Value> {
        let vars = self.variables.lock().unwrap();
        vars.get(name).cloned()
    }
}

pub struct Runtime {
    pub env: Environment,
    pub commands: Vec<Command>,
    pub events: Vec<Event>,
    pub schedules: Vec<Schedule>,
    pub middleware: Vec<MiddlewareDef>,
    pub context_menus: Vec<ContextMenu>,
    pub types: Vec<CustomType>,
    pub tests: Vec<TestCase>,
}

impl Runtime {
    pub fn new(program: Program) -> Self {
        let env = Environment::new();
        Self {
            env,
            commands: program.commands,
            events: program.events,
            schedules: program.schedules,
            middleware: program.middleware,
            context_menus: program.context_menus,
            types: program.types,
            tests: program.tests,
        }
    }

    pub fn register_globals(&self) {
        // Register Discord.js-like globals
        self.env.set("true", Value::Boolean(true));
        self.env.set("false", Value::Boolean(false));
        self.env.set("null", Value::Null);
    }

    pub fn execute(&self, statements: &[Statement]) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();

        for stmt in statements {
            let result = self.execute_statement(stmt)?;
            results.push(result);
        }

        Ok(results)
    }

    fn execute_statement(&self, stmt: &Statement) -> Result<Value, String> {
        match stmt {
            Statement::Reply { content, ephemeral, components } => {
                let text = self.evaluate_pattern(content)?;
                // TODO: send reply through Discord
                println!("[Reply] {:?} (ephemeral: {:?}, components: {:?})", text, ephemeral, components);
                Ok(text)
            }
            Statement::Send { content, channel } => {
                let text = self.evaluate_pattern(content)?;
                let ch = channel.as_ref()
                    .and_then(|c| self.evaluate_pattern(c).ok())
                    .unwrap_or_else(|| Value::String("general".into()));
                println!("[Send] {:?} to {:?}", text, ch);
                Ok(text)
            }
            Statement::Embed(embed) => {
                // Store embed for later use
                Ok(Value::EmbedData(embed.clone()))
            }
            Statement::Let { name, value } => {
                let val = self.evaluate_pattern(value)?;
                self.env.set(name, val.clone());
                Ok(val)
            }
            Statement::Set { target, value } => {
                let val = self.evaluate_pattern(value)?;
                if let Pattern::Variable(name) = target {
                    self.env.set(name, val.clone());
                }
                Ok(val)
            }
            Statement::If { condition, body, elifs, else_body } => {
                let cond_val = self.evaluate_pattern(condition)?;
                if self.is_truthy(&cond_val) {
                    self.execute(body).map(|mut v| v.pop().unwrap_or(Value::Null))
                } else {
                    let mut executed = false;
                    for (elif_cond, elif_body) in elifs {
                        if !executed {
                            let ec = self.evaluate_pattern(elif_cond)?;
                            if self.is_truthy(&ec) {
                                let _ = self.execute(elif_body)?;
                                executed = true;
                            }
                        }
                    }
                    if !executed {
                        if let Some(eb) = else_body {
                            let _ = self.execute(eb)?;
                        }
                    }
                    Ok(Value::Null)
                }
            }
            Statement::For { var, iterable, body } => {
                let iter_val = self.evaluate_pattern(iterable)?;
                let items = match iter_val {
                    Value::List(list) => list,
                    _ => vec![],
                };
                for item in items {
                    self.env.set(var, item);
                    self.execute(body)?;
                }
                Ok(Value::Null)
            }
            Statement::TryCatch { try_body, catch_var, catch_body, finally_body } => {
                let result = self.execute(try_body);
                if let Err(ref err) = result {
                    if let Some(var) = catch_var {
                        self.env.set(var, Value::String(err.clone()));
                    }
                    self.execute(catch_body)?;
                }
                if let Some(fb) = finally_body {
                    self.execute(fb)?;
                }
                match result {
                    Ok(values) => Ok(values.last().cloned().unwrap_or(Value::Null)),
                    Err(_) => Ok(Value::Null),
                }
            }
            Statement::Throw { message } => {
                let msg = self.evaluate_pattern(message)?;
                let text = self.value_to_string(&msg);
                return Err(text);
            }
            Statement::Cancel => {
                return Err("CANCELLED".into());
            }
            Statement::Log { level, message, channel } => {
                let msg = self.evaluate_pattern(message)?;
                let text = self.value_to_string(&msg);
                println!("[{}] {} {:?}", level.to_uppercase(), text, channel);
                Ok(Value::Null)
            }
            Statement::StdCall { module, func, args, result_var } => {
                let evaluated_args: Vec<Value> = args.iter()
                    .map(|a| self.evaluate_pattern(a))
                    .collect::<Result<Vec<_>, _>>()?;
                let result = self.execute_stdlib(module, func, &evaluated_args);
                if let Some(var) = result_var {
                    if let Ok(ref val) = result {
                        self.env.set(var, val.clone());
                    }
                }
                result
            }
            Statement::DiscordAction { action, args } => {
                let evaluated_args: Vec<Value> = args.iter()
                    .map(|a| self.evaluate_pattern(a))
                    .collect::<Result<Vec<_>, _>>()?;
                println!("[Discord] {} {:?}", action, evaluated_args);
                Ok(Value::Null)
            }
            Statement::DBQuery { query, params, result_var } => {
                let evaluated_params: Vec<String> = params.iter()
                    .map(|p| self.evaluate_pattern(p).map(|v| self.value_to_string(&v)))
                    .collect::<Result<Vec<_>, _>>()?;
                println!("[DB] {} {:?}", query, evaluated_params);
                if let Some(var) = result_var {
                    self.env.set(var, Value::List(vec![]));
                }
                Ok(Value::List(vec![]))
            }
            Statement::HTTPRequest { method, url, headers, body, result_var } => {
                let url_val = self.evaluate_pattern(url)?;
                let url_str = self.value_to_string(&url_val);
                println!("[HTTP] {} {}", method.to_uppercase(), url_str);
                if let Some(var) = result_var {
                    self.env.set(var, Value::Object(HashMap::new()));
                }
                Ok(Value::Object(HashMap::new()))
            }
            Statement::JSONOp { op, key, value, result_var } => {
                let key_val = self.evaluate_pattern(key)?;
                let key_str = self.value_to_string(&key_val);
                let val = value.as_ref().map(|v| self.evaluate_pattern(v)).transpose()?;
                println!("[JSON] {} {} {:?}", op, key_str, val);
                if let Some(var) = result_var {
                    self.env.set(var, Value::Null);
                }
                Ok(Value::Null)
            }
            Statement::FileOp { op, path, content, result_var } => {
                let path_val = self.evaluate_pattern(path)?;
                let path_str = self.value_to_string(&path_val);
                println!("[File] {} {}", op, path_str);
                if let Some(var) = result_var {
                    self.env.set(var, Value::String(String::new()));
                }
                Ok(Value::Null)
            }
            Statement::VoiceOp { op, args } => {
                println!("[Voice] {} {:?}", op, args);
                Ok(Value::Null)
            }
            Statement::WebhookOp { op, url, args } => {
                let url_val = self.evaluate_pattern(url)?;
                let url_str = self.value_to_string(&url_val);
                println!("[Webhook] {} {}", op, url_str);
                Ok(Value::Null)
            }
            Statement::CacheOp { op, key, value, ttl, result_var } => {
                let key_val = self.evaluate_pattern(key)?;
                let key_str = self.value_to_string(&key_val);
                println!("[Cache] {} {} (ttl: {:?})", op, key_str, ttl);
                if let Some(var) = result_var {
                    self.env.set(var, Value::Null);
                }
                Ok(Value::Null)
            }
            Statement::Thread { body } => {
                self.execute(body).map(|mut v| v.pop().unwrap_or(Value::Null))
            }
            Statement::Parallel { branches, result_var } => {
                let results: Vec<Value> = branches.iter()
                    .map(|b| self.execute(b).unwrap_or_default().last().cloned().unwrap_or(Value::Null))
                    .collect();
                let result = Value::List(results);
                if let Some(var) = result_var {
                    self.env.set(var, result.clone());
                }
                Ok(result)
            }
            Statement::Await { expr, result_var } => {
                let result = self.evaluate_pattern(expr)?;
                if let Some(var) = result_var {
                    self.env.set(var, result.clone());
                }
                Ok(result)
            }
            Statement::Autocomplete { options } => {
                let opts = self.evaluate_pattern(options)?;
                println!("[Autocomplete] {:?}", opts);
                Ok(Value::Null)
            }
            Statement::Paginate { items, per_page, embed_template, buttons, timeout } => {
                let items_val = self.evaluate_pattern(items)?;
                let item_list = match items_val {
                    Value::List(list) => list,
                    _ => vec![],
                };
                let pages = (item_list.len() as f64 / *per_page as f64).ceil() as i64;
                println!("[Pagination] {} items, {} pages, {} per page", item_list.len(), pages, per_page);
                Ok(Value::Null)
            }
            Statement::Row(row) => {
                println!("[Row] {} components", row.components.len());
                Ok(Value::Null)
            }
            Statement::Modal(modal) => {
                println!("[Modal] {}", modal.title);
                Ok(Value::Null)
            }
            Statement::Expression(expr) => {
                self.evaluate_pattern(expr)
            }
            Statement::Return { value } => {
                if let Some(val) = value {
                    self.evaluate_pattern(val)
                } else {
                    Ok(Value::Null)
                }
            }
            Statement::CustomEvent { name, args } => {
                println!("[Event] {} {:?}", name, args);
                Ok(Value::Null)
            }
            Statement::Mock { name, fields } => {
                let mut map = HashMap::new();
                for (k, v) in fields {
                    let val = self.evaluate_pattern(v)?;
                    map.insert(k.clone(), val);
                }
                println!("[Mock] {} = {:?}", name, map);
                Ok(Value::Object(map))
            }
            Statement::Assert { condition, message } => {
                let result = self.evaluate_pattern(condition)?;
                let truthy = self.is_truthy(&result);
                if !truthy {
                    let msg = message.clone().unwrap_or_else(|| "Assertion failed".into());
                    return Err(msg);
                }
                Ok(Value::Boolean(true))
            }
            Statement::CancelEvent => {
                println!("[CancelEvent]");
                Ok(Value::Null)
            }
            Statement::Execute { command, result_var } => {
                println!("[Execute] command '{}'", command);
                if let Some(var) = result_var {
                    self.env.set(var, Value::Null);
                }
                Ok(Value::Null)
            }
            Statement::FnCall { name, args, result_var } => {
                println!("[FnCall] {} with {} args", name, args.len());
                if let Some(var) = result_var {
                    self.env.set(var, Value::Null);
                }
                Ok(Value::Null)
            }
            Statement::While { condition, body } => {
                while self.is_truthy(&self.evaluate_pattern(condition)?) {
                    self.execute(body)?;
                }
                Ok(Value::Null)
            }
            Statement::Break | Statement::Continue => Ok(Value::Null),
            Statement::Match { value, arms, else_body } => {
                let val = self.evaluate_pattern(value)?;
                let mut matched = false;
                for (pat, arm_body) in arms {
                    let pat_val = self.evaluate_pattern(pat)?;
                    if val == pat_val {
                        self.execute(arm_body)?;
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    if let Some(eb) = else_body {
                        self.execute(eb)?;
                    }
                }
                Ok(Value::Null)
            }
            Statement::TryExpr { body, catch_var, catch_body } => {
                match self.execute_statement(body) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        if let Some(var) = catch_var {
                            self.env.set(var, Value::String(e.clone()));
                        }
                        self.execute_statement(catch_body)
                    }
                }
            }
        }
    }

    fn evaluate_pattern(&self, pattern: &Pattern) -> Result<Value, String> {
        match pattern {
            Pattern::Literal(lit) => Ok(self.literal_to_value(lit)),
            Pattern::Variable(name) => {
                self.env.get(name).ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Pattern::MemberAccess(obj, field) => {
                let obj_val = self.evaluate_pattern(obj)?;
                match obj_val {
                    Value::Object(map) => {
                        map.get(field).cloned()
                            .ok_or_else(|| format!("Field '{}' not found", field))
                    }
                    Value::DiscordUser(_, _) => {
                        match field.as_str() {
                            "id" => Ok(Value::String(String::new())),
                            "name" => Ok(Value::String(String::new())),
                            "tag" => Ok(Value::String(String::new())),
                            "mention" => Ok(Value::String(String::new())),
                            _ => Ok(Value::Null),
                        }
                    }
                    _ => Ok(Value::Null),
                }
            }
            Pattern::Index(obj, index) => {
                let obj_val = self.evaluate_pattern(obj)?;
                let idx_val = self.evaluate_pattern(index)?;
                match (obj_val, idx_val) {
                    (Value::List(list), Value::Integer(i)) => {
                        list.get(i as usize).cloned().ok_or_else(|| format!("Index out of bounds: {}", i))
                    }
                    _ => Ok(Value::Null),
                }
            }
            Pattern::Call(func, args) => {
                let func_name = match func.as_ref() {
                    Pattern::Variable(name) => name.clone(),
                    Pattern::MemberAccess(obj, field) => {
                        let obj_val = self.evaluate_pattern(obj)?;
                        let obj_str = self.value_to_string(&obj_val);
                        format!("{}.{}", obj_str, field)
                    }
                    _ => return Err("Cannot call non-function".into()),
                };

                let evaluated_args: Vec<Value> = args.iter()
                    .map(|a| self.evaluate_pattern(a))
                    .collect::<Result<Vec<_>, _>>()?;

                // Handle env() calls
                if func_name == "env" {
                    if let Some(Value::String(key)) = evaluated_args.first() {
                        let val = std::env::var(key).unwrap_or_default();
                        return Ok(Value::String(val));
                    }
                    return Ok(Value::Null);
                }

                // Handle stdlib calls
                if func_name.starts_with("std.") || func_name.starts_with("std::") {
                    let parts: Vec<&str> = func_name.split(|c| c == '.' || c == ':').collect();
                    if parts.len() >= 2 {
                        return self.execute_stdlib(parts[0], parts[1], &evaluated_args);
                    }
                }

                // Handle discord.* calls
                if func_name.starts_with("discord.") || func_name == "discord" {
                    println!("[Discord Call] {} {:?}", func_name, evaluated_args);
                    return Ok(Value::Null);
                }

                Ok(Value::Null)
            }
            Pattern::BinOp(left, op, right) => {
                let l = self.evaluate_pattern(left)?;
                let r = self.evaluate_pattern(right)?;
                self.evaluate_binop(&l, op, &r)
            }
            Pattern::List(items) => {
                let vals: Result<Vec<Value>, String> = items.iter()
                    .map(|i| self.evaluate_pattern(i))
                    .collect();
                Ok(Value::List(vals?))
            }
            Pattern::Object(fields) => {
                let mut map = HashMap::new();
                for (k, v) in fields {
                    let val = self.evaluate_pattern(v)?;
                    map.insert(k.clone(), val);
                }
                Ok(Value::Object(map))
            }
            Pattern::Interpolated(parts) => {
                let mut result = String::new();
                for part in parts {
                    let val = self.evaluate_pattern(part)?;
                    result.push_str(&self.value_to_string(&val));
                }
                Ok(Value::String(result))
            }
            Pattern::EnumVariant(_enum_name, variant_name, _fields) => {
                Ok(Value::String(format!("{}::{}", _enum_name, variant_name)))
            }
        }
    }

    fn execute_stdlib(&self, module: &str, func: &str, args: &[Value]) -> Result<Value, String> {
        match func {
            // String functions
            "upper" => {
                if let Some(Value::String(s)) = args.first() {
                    Ok(Value::String(s.to_uppercase()))
                } else { Ok(Value::Null) }
            }
            "lower" => {
                if let Some(Value::String(s)) = args.first() {
                    Ok(Value::String(s.to_lowercase()))
                } else { Ok(Value::Null) }
            }
            "len" => {
                match args.first() {
                    Some(Value::String(s)) => Ok(Value::Integer(s.len() as i64)),
                    Some(Value::List(l)) => Ok(Value::Integer(l.len() as i64)),
                    _ => Ok(Value::Integer(0)),
                }
            }
            "slice" => {
                if args.len() >= 2 {
                    if let Some(Value::String(s)) = args.first() {
                        let start = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(0);
                        let end = args.get(2).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(s.len());
                        if start < s.len() && end <= s.len() {
                            return Ok(Value::String(s[start..end].to_string()));
                        }
                    }
                }
                Ok(Value::Null)
            }
            "join" => {
                if let Some(Value::List(list)) = args.first() {
                    let sep = args.get(1).and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default();
                    let strs: Vec<String> = list.iter().map(|v| self.value_to_string(v)).collect();
                    Ok(Value::String(strs.join(&sep)))
                } else { Ok(Value::Null) }
            }
            "contains" => {
                match args.first() {
                    Some(Value::String(s)) => {
                        let sub = args.get(1).and_then(|v| if let Value::String(ss) = v { Some(ss) } else { None });
                        Ok(Value::Boolean(sub.map_or(false, |sub| s.contains(sub))))
                    }
                    Some(Value::List(list)) => {
                        let item = args.get(1);
                        Ok(Value::Boolean(item.map_or(false, |item| list.contains(item))))
                    }
                    _ => Ok(Value::Boolean(false)),
                }
            }
            "replace" => {
                if let Some(Value::String(s)) = args.first() {
                    let from = args.get(1).and_then(|v| if let Value::String(f) = v { Some(f.clone()) } else { None }).unwrap_or_default();
                    let to = args.get(2).and_then(|v| if let Value::String(t) = v { Some(t.clone()) } else { None }).unwrap_or_default();
                    Ok(Value::String(s.replace(&from, &to)))
                } else { Ok(Value::Null) }
            }
            "trim" => {
                if let Some(Value::String(s)) = args.first() {
                    Ok(Value::String(s.trim().to_string()))
                } else { Ok(Value::Null) }
            }
            "split" => {
                if let Some(Value::String(s)) = args.first() {
                    let sep = args.get(1).and_then(|v| if let Value::String(ss) = v { Some(ss.clone()) } else { None }).unwrap_or_default();
                    let parts: Vec<Value> = s.split(&sep).map(|p| Value::String(p.to_string())).collect();
                    Ok(Value::List(parts))
                } else { Ok(Value::Null) }
            }
            // Math functions
            "random" => {
                let min = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
                let max = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(100);
                use rand::Rng;
                let val: i64 = rand::thread_rng().gen_range(min..=max);
                Ok(Value::Integer(val))
            }
            "clamp" => {
                let val = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
                let min = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
                let max = args.get(2).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(100.0);
                Ok(Value::Float(val.max(min).min(max)))
            }
            "abs" => {
                let val = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
                Ok(Value::Float(val.abs()))
            }
            "min" => {
                let a = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
                let b = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
                Ok(Value::Integer(a.min(b)))
            }
            "max" => {
                let a = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
                let b = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
                Ok(Value::Integer(a.max(b)))
            }
            "round" | "floor" | "ceil" => {
                let val = args.get(0).and_then(|v| if let Value::Float(f) = v { Some(*f) } else if let Value::Integer(i) = v { Some(*i as f64) } else { None }).unwrap_or(0.0);
                match func {
                    "round" => Ok(Value::Integer(val.round() as i64)),
                    "floor" => Ok(Value::Integer(val.floor() as i64)),
                    "ceil" => Ok(Value::Integer(val.ceil() as i64)),
                    _ => Ok(Value::Integer(val as i64)),
                }
            }
            // Time functions
            "now" => {
                let now = chrono::Utc::now().timestamp();
                Ok(Value::Integer(now))
            }
            "format_time" => {
                // TODO: implement format
                Ok(Value::String(String::new()))
            }
            "sleep" => {
                if let Some(Value::Integer(ms)) = args.first() {
                    std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                }
                Ok(Value::Null)
            }
            // Collection functions
            "push" => {
                if let Some(Value::List(list)) = args.first() {
                    let mut new_list = list.clone();
                    if let Some(item) = args.get(1) {
                        new_list.push(item.clone());
                    }
                    Ok(Value::List(new_list))
                } else { Ok(Value::Null) }
            }
            "pop" => {
                if let Some(Value::List(list)) = args.first() {
                    let mut new_list = list.clone();
                    let last = new_list.pop();
                    Ok(last.unwrap_or(Value::Null))
                } else { Ok(Value::Null) }
            }
            "first" => {
                if let Some(Value::List(list)) = args.first() {
                    Ok(list.first().cloned().unwrap_or(Value::Null))
                } else { Ok(Value::Null) }
            }
            "last" => {
                if let Some(Value::List(list)) = args.first() {
                    Ok(list.last().cloned().unwrap_or(Value::Null))
                } else { Ok(Value::Null) }
            }
            "filter" | "map" => {
                if let Some(Value::List(list)) = args.first() {
                    // TODO: implement with function argument
                    Ok(Value::List(list.clone()))
                } else { Ok(Value::Null) }
            }
            "chunk" => {
                if let Some(Value::List(list)) = args.first() {
                    let size = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(1);
                    let chunks: Vec<Value> = list.chunks(size).map(|c| Value::List(c.to_vec())).collect();
                    Ok(Value::List(chunks))
                } else { Ok(Value::Null) }
            }
            // Discord helpers
            "avatar_url" | "hex_to_int" | "channel_name" | "role_name" | "tag" => {
                Ok(Value::Null)
            }
            _ => Err(format!("Unknown stdlib function: {}", func)),
        }
    }

    fn evaluate_binop(&self, left: &Value, op: &BinOp, right: &Value) -> Result<Value, String> {
        match op {
            BinOp::Add => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 + b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a + *b as f64)),
                    (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), b) => Ok(Value::String(format!("{}{}", a, self.value_to_string(b)))),
                    (a, Value::String(b)) => Ok(Value::String(format!("{}{}", self.value_to_string(a), b))),
                    _ => Ok(Value::Integer(0)),
                }
            }
            BinOp::Sub => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a - *b as f64)),
                    _ => Ok(Value::Integer(0)),
                }
            }
            BinOp::Mul => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 * b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a * *b as f64)),
                    _ => Ok(Value::Integer(0)),
                }
            }
            BinOp::Div => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a / (*b).max(1))),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
                    (Value::Integer(a), Value::Float(b)) => Ok(Value::Float(*a as f64 / b)),
                    (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a / *b as f64)),
                    _ => Ok(Value::Integer(0)),
                }
            }
            BinOp::Mod => {
                if let (Value::Integer(a), Value::Integer(b)) = (left, right) {
                    Ok(Value::Integer(a % (*b).max(1)))
                } else { Ok(Value::Integer(0)) }
            }
            BinOp::Eq => Ok(Value::Boolean(left == right)),
            BinOp::Neq => Ok(Value::Boolean(left != right)),
            BinOp::Lt => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
                    _ => Ok(Value::Boolean(false)),
                }
            }
            BinOp::Gt => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
                    _ => Ok(Value::Boolean(false)),
                }
            }
            BinOp::Lte => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                    _ => Ok(Value::Boolean(false)),
                }
            }
            BinOp::Gte => {
                match (left, right) {
                    (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                    (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                    _ => Ok(Value::Boolean(false)),
                }
            }
            BinOp::And => {
                let lb = self.is_truthy(left);
                let rb = self.is_truthy(right);
                Ok(Value::Boolean(lb && rb))
            }
            BinOp::Or => {
                let lb = self.is_truthy(left);
                let rb = self.is_truthy(right);
                Ok(Value::Boolean(lb || rb))
            }
            BinOp::Assign => {
                // Handled in parse_assignment
                Ok(right.clone())
            }
        }
    }

    fn literal_to_value(&self, lit: &Literal) -> Value {
        match lit {
            Literal::String(s, _) => Value::String(s.clone()),
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Float(f) => Value::Float(*f),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Null => Value::Null,
        }
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Boolean(b) => *b,
            Value::Integer(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Object(m) => !m.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }

    fn value_to_string(&self, val: &Value) -> String {
        match val {
            Value::String(s) => s.clone(),
            Value::Integer(i) => i.to_string(),
            Value::Float(f) => f.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".into(),
            Value::List(l) => format!("[{}]", l.iter().map(|v| self.value_to_string(v)).collect::<Vec<_>>().join(", ")),
            Value::Object(m) => format!("{{{}}}", m.iter().map(|(k, v)| format!("{}: {}", k, self.value_to_string(v))).collect::<Vec<_>>().join(", ")),
            Value::Function(name, _) => format!("<function {}>", name),
            Value::DiscordUser(id, _name) => format!("<@{}>", id),
            Value::DiscordChannel(_id, name) => format!("#{}", name),
            Value::DiscordRole(_id, name) => format!("@{}", name),
            Value::DiscordMember(id, _name, _) => format!("<@{}>", id),
            Value::EmbedData(_) => "<embed>".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_basics() {
        let env = Environment::new();
        env.set("x", Value::Integer(42));
        assert_eq!(env.get("x"), Some(Value::Integer(42)));
        assert_eq!(env.get("nonexistent"), None);
    }

    #[test]
    fn test_environment_overwrite() {
        let env = Environment::new();
        env.set("x", Value::Integer(1));
        env.set("x", Value::String("two".into()));
        assert_eq!(env.get("x"), Some(Value::String("two".into())));
    }

    fn empty_program() -> Program {
        Program {
            config: None,
            imports: vec![],
            commands: vec![],
            events: vec![],
            schedules: vec![],
            middleware: vec![],
            context_menus: vec![],
            types: vec![],
            tests: vec![],
            functions: vec![],
            enums: vec![],
            migrations: vec![],
            models: vec![],
        }
    }

    #[test]
    fn test_literal_to_value() {
        let rt = Runtime::new(empty_program());
        assert_eq!(rt.literal_to_value(&Literal::Integer(42)), Value::Integer(42));
        assert_eq!(rt.literal_to_value(&Literal::String("hello".into(), false)), Value::String("hello".into()));
        assert_eq!(rt.literal_to_value(&Literal::Float(3.14)), Value::Float(3.14));
        assert_eq!(rt.literal_to_value(&Literal::Boolean(true)), Value::Boolean(true));
        assert_eq!(rt.literal_to_value(&Literal::Null), Value::Null);
    }

    #[test]
    fn test_is_truthy() {
        let rt = Runtime::new(empty_program());
        assert!(rt.is_truthy(&Value::Boolean(true)));
        assert!(!rt.is_truthy(&Value::Boolean(false)));
        assert!(rt.is_truthy(&Value::Integer(1)));
        assert!(!rt.is_truthy(&Value::Integer(0)));
        assert!(rt.is_truthy(&Value::String("hi".into())));
        assert!(!rt.is_truthy(&Value::String("".into())));
        assert!(rt.is_truthy(&Value::List(vec![Value::Integer(1)])));
        assert!(!rt.is_truthy(&Value::List(vec![])));
        assert!(!rt.is_truthy(&Value::Null));
    }

    #[test]
    fn test_evaluate_literal() {
        let rt = Runtime::new(empty_program());
        assert_eq!(rt.evaluate_pattern(&Pattern::Literal(Literal::Integer(42))).unwrap(), Value::Integer(42));
    }

    #[test]
    fn test_execute_set_statement() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let stmt = Statement::Set {
            target: Pattern::Variable("x".into()),
            value: Pattern::Literal(Literal::Integer(42)),
        };
        let result = rt.execute_statement(&stmt);
        assert!(result.is_ok());
        assert_eq!(rt.env.get("x"), Some(Value::Integer(42)));
    }

    #[test]
    fn test_execute_expression() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let stmt = Statement::Expression(
            Pattern::BinOp(
                Box::new(Pattern::Literal(Literal::Integer(2))),
                BinOp::Add,
                Box::new(Pattern::Literal(Literal::Integer(3))),
            )
        );
        let result = rt.execute_statement(&stmt);
        assert_eq!(result, Ok(Value::Integer(5)));
    }

    #[test]
    fn test_let_statement() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let stmt = Statement::Let {
            name: "x".into(),
            value: Pattern::Literal(Literal::String("hello".into(), false)),
        };
        let result = rt.execute_statement(&stmt);
        assert!(result.is_ok());
        assert_eq!(rt.env.get("x"), Some(Value::String("hello".into())));
    }

    #[test]
    fn test_if_truthy() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let body = vec![
            Statement::Set {
                target: Pattern::Variable("result".into()),
                value: Pattern::Literal(Literal::Integer(1)),
            }
        ];
        let stmt = Statement::If {
            condition: Pattern::Literal(Literal::Boolean(true)),
            body,
            elifs: vec![],
            else_body: None,
        };
        let result = rt.execute_statement(&stmt);
        assert!(result.is_ok());
        assert_eq!(rt.env.get("result"), Some(Value::Integer(1)));
    }

    #[test]
    fn test_log_statement() {
        let rt = Runtime::new(empty_program());

        let stmt = Statement::Log {
            level: "info".into(),
            message: Pattern::Literal(Literal::String("test log".into(), false)),
            channel: None,
        };
        let result = rt.execute_statement(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_for_loop() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let stmt = Statement::For {
            var: "item".into(),
            iterable: Pattern::List(vec![
                Pattern::Literal(Literal::Integer(1)),
                Pattern::Literal(Literal::Integer(2)),
                Pattern::Literal(Literal::Integer(3)),
            ]),
            body: vec![
                Statement::Log {
                    level: "debug".into(),
                    message: Pattern::Variable("item".into()),
                    channel: None,
                }
            ],
        };
        let result = rt.execute_statement(&stmt);
        assert!(result.is_ok());
    }

    #[test]
    fn test_return_statement() {
        let rt = Runtime::new(empty_program());
        rt.register_globals();

        let stmt = Statement::Return { value: Some(Pattern::Literal(Literal::Integer(99))) };
        let result = rt.execute_statement(&stmt);
        assert_eq!(result, Ok(Value::Integer(99)));
    }
}
