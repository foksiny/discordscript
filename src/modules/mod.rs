use crate::runtime::Value;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

pub struct Module {
    pub name: String,
    pub functions: HashMap<String, ModuleFunction>,
}

pub type ModuleFunction = fn(&[Value]) -> Result<Value, String>;

pub struct ModuleRegistry {
    pub modules: HashMap<String, Module>,
}

static JSON_STORE: LazyLock<Mutex<HashMap<String, Value>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static CACHE_STORE: LazyLock<Mutex<HashMap<String, (Value, Option<Instant>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        registry.register_http_module();
        registry.register_db_module();
        registry.register_json_module();
        registry.register_cache_module();
        registry
    }

    fn register_http_module(&mut self) {
        let mut functions: HashMap<String, ModuleFunction> = HashMap::new();
        functions.insert("get".into(), http_get);
        functions.insert("post".into(), http_post);
        functions.insert("put".into(), http_put);
        functions.insert("delete".into(), http_delete);
        functions.insert("stream".into(), http_stream);
        self.modules.insert("http".into(), Module { name: "http".into(), functions });
    }

    fn register_db_module(&mut self) {
        let mut functions: HashMap<String, ModuleFunction> = HashMap::new();
        functions.insert("query".into(), db_query);
        functions.insert("find".into(), db_find);
        functions.insert("find_all".into(), db_find_all);
        functions.insert("insert".into(), db_insert);
        functions.insert("update".into(), db_update);
        functions.insert("delete".into(), db_delete);
        self.modules.insert("db".into(), Module { name: "db".into(), functions });
    }

    fn register_json_module(&mut self) {
        let mut functions: HashMap<String, ModuleFunction> = HashMap::new();
        functions.insert("set".into(), json_set);
        functions.insert("get".into(), json_get);
        functions.insert("delete".into(), json_delete);
        functions.insert("push".into(), json_push);
        functions.insert("save".into(), json_save);
        self.modules.insert("json".into(), Module { name: "json".into(), functions });
    }

    fn register_cache_module(&mut self) {
        let mut functions: HashMap<String, ModuleFunction> = HashMap::new();
        functions.insert("set".into(), cache_set);
        functions.insert("get".into(), cache_get);
        functions.insert("delete".into(), cache_delete);
        functions.insert("clear".into(), cache_clear);
        functions.insert("stats".into(), cache_stats);
        self.modules.insert("cache".into(), Module { name: "cache".into(), functions });
    }

    pub fn call(&self, module: &str, func: &str, args: &[Value]) -> Result<Value, String> {
        if let Some(mod_) = self.modules.get(module) {
            if let Some(f) = mod_.functions.get(func) {
                f(args)
            } else {
                Err(format!("Unknown function '{}' in module '{}'", func, module))
            }
        } else {
            Err(format!("Unknown module '{}'", module))
        }
    }
}

// HTTP module (print-only stubs)
fn http_get(_args: &[Value]) -> Result<Value, String> {
    println!("[HTTP] GET request simulated");
    Ok(Value::Object(HashMap::new()))
}

fn http_post(_args: &[Value]) -> Result<Value, String> {
    println!("[HTTP] POST request simulated");
    Ok(Value::Object(HashMap::new()))
}

fn http_put(_args: &[Value]) -> Result<Value, String> {
    println!("[HTTP] PUT request simulated");
    Ok(Value::Object(HashMap::new()))
}

fn http_delete(_args: &[Value]) -> Result<Value, String> {
    println!("[HTTP] DELETE request simulated");
    Ok(Value::Null)
}

fn http_stream(_args: &[Value]) -> Result<Value, String> {
    println!("[HTTP] STREAM request simulated");
    Ok(Value::Null)
}

// DB module (print-only stubs — real rusqlite would need a connection manager)
fn db_query(args: &[Value]) -> Result<Value, String> {
    if let Some(first) = args.first() {
        let query = value_to_debug_string(first);
        println!("[DB] Query: {}", query);
    }
    Ok(Value::List(vec![]))
}

fn db_find(args: &[Value]) -> Result<Value, String> {
    println!("[DB] Find simulated");
    Ok(Value::Null)
}

fn db_find_all(args: &[Value]) -> Result<Value, String> {
    println!("[DB] Find all simulated");
    Ok(Value::List(vec![]))
}

fn db_insert(args: &[Value]) -> Result<Value, String> {
    println!("[DB] Insert simulated");
    Ok(Value::Null)
}

fn db_update(args: &[Value]) -> Result<Value, String> {
    println!("[DB] Update simulated");
    Ok(Value::Null)
}

fn db_delete(args: &[Value]) -> Result<Value, String> {
    println!("[DB] Delete simulated");
    Ok(Value::Null)
}

// JSON module (in-memory)
fn json_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("json.set requires at least 2 arguments (key, value)".into());
    }
    let key = value_to_debug_string(&args[0]);
    let value = args[1].clone();
    let mut store = JSON_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.insert(key, value);
    Ok(Value::Null)
}

fn json_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("json.get requires a key argument".into());
    }
    let key = value_to_debug_string(&args[0]);
    let store = JSON_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    Ok(store.get(&key).cloned().unwrap_or(Value::Null))
}

fn json_delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("json.delete requires a key argument".into());
    }
    let key = value_to_debug_string(&args[0]);
    let mut store = JSON_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.remove(&key);
    Ok(Value::Null)
}

fn json_push(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("json.push requires 2 arguments (key, value)".into());
    }
    let key = value_to_debug_string(&args[0]);
    let value = args[1].clone();
    let mut store = JSON_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let entry = store.entry(key).or_insert_with(|| Value::List(vec![]));
    match entry {
        Value::List(ref mut list) => list.push(value),
        _ => return Err("json.push target is not a list".into()),
    }
    Ok(Value::Null)
}

fn json_save(_args: &[Value]) -> Result<Value, String> {
    println!("[JSON] In-memory store active (no persistence)");
    Ok(Value::Null)
}

// Cache module (in-memory with TTL)
fn cache_set(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("cache.set requires at least 2 arguments (key, value)".into());
    }
    let key = value_to_debug_string(&args[0]);
    let value = args[1].clone();
    let ttl = if args.len() > 2 {
        if let Value::Integer(secs) = args[2] {
            Some(Instant::now() + std::time::Duration::from_secs(secs as u64))
        } else { None }
    } else { None };
    let mut store = CACHE_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.insert(key, (value, ttl));
    Ok(Value::Null)
}

fn cache_get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("cache.get requires a key argument".into());
    }
    let key = value_to_debug_string(&args[0]);
    let mut store = CACHE_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    if let Some((value, ttl)) = store.get(&key) {
        if let Some(expiry) = ttl {
            if Instant::now() > *expiry {
                store.remove(&key);
                return Ok(Value::Null);
            }
        }
        Ok(value.clone())
    } else {
        Ok(Value::Null)
    }
}

fn cache_delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("cache.delete requires a key argument".into());
    }
    let key = value_to_debug_string(&args[0]);
    let mut store = CACHE_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.remove(&key);
    Ok(Value::Null)
}

fn cache_clear(_args: &[Value]) -> Result<Value, String> {
    let mut store = CACHE_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    store.clear();
    Ok(Value::Null)
}

fn cache_stats(_args: &[Value]) -> Result<Value, String> {
    let store = CACHE_STORE.lock().map_err(|e| format!("Lock error: {}", e))?;
    let mut obj = HashMap::new();
    obj.insert("size".into(), Value::Integer(store.len() as i64));
    Ok(Value::Object(obj))
}

fn value_to_debug_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".into(),
        Value::List(_) => "[list]".into(),
        Value::Object(_) => "{object}".into(),
        Value::DiscordUser(id, _) => format!("<@{}>", id),
        Value::DiscordChannel(id, _) => format!("<#{}>", id),
        Value::DiscordRole(id, _) => format!("<@&{}>", id),
        Value::DiscordMember(id, _, _) => format!("<@{}>", id),
        Value::EmbedData(_) => "{embed}".into(),
        Value::Function(_, _) => "<function>".into(),
    }
}
