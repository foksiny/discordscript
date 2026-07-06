use crate::runtime::Value;
use std::collections::HashMap;

pub struct StdLib {
    pub functions: HashMap<String, StdFunction>,
}

pub type StdFunction = fn(&[Value]) -> Result<Value, String>;

impl StdLib {
    pub fn new() -> Self {
        let mut functions: HashMap<String, StdFunction> = HashMap::new();

        // String functions
        functions.insert("upper".into(), std_upper);
        functions.insert("lower".into(), std_lower);
        functions.insert("len".into(), std_len);
        functions.insert("slice".into(), std_slice);
        functions.insert("join".into(), std_join);
        functions.insert("contains".into(), std_contains);
        functions.insert("replace".into(), std_replace);
        functions.insert("trim".into(), std_trim);
        functions.insert("split".into(), std_split);

        // Math functions
        functions.insert("random".into(), std_random);
        functions.insert("clamp".into(), std_clamp);
        functions.insert("abs".into(), std_abs);
        functions.insert("min".into(), std_min);
        functions.insert("max".into(), std_max);
        functions.insert("round".into(), std_round);
        functions.insert("floor".into(), std_floor);
        functions.insert("ceil".into(), std_ceil);

        // Time functions
        functions.insert("now".into(), std_now);
        functions.insert("sleep".into(), std_sleep);

        // Collection functions
        functions.insert("push".into(), std_push);
        functions.insert("pop".into(), std_pop);
        functions.insert("first".into(), std_first);
        functions.insert("last".into(), std_last);
        functions.insert("includes".into(), std_includes);
        functions.insert("chunk".into(), std_chunk);
        functions.insert("filter".into(), std_filter);

        Self { functions }
    }

    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, String> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(format!("Unknown stdlib function: {}", name))
        }
    }
}

fn std_upper(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::String(s)) => Value::String(s.to_uppercase()),
        _ => Value::Null,
    })
}

fn std_lower(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::String(s)) => Value::String(s.to_lowercase()),
        _ => Value::Null,
    })
}

fn std_len(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::String(s)) => Value::Integer(s.len() as i64),
        Some(Value::List(l)) => Value::Integer(l.len() as i64),
        _ => Value::Integer(0),
    })
}

fn std_slice(args: &[Value]) -> Result<Value, String> {
    if args.len() >= 2 {
        if let Some(Value::String(s)) = args.first() {
            let start = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(0);
            let end = args.get(2).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(s.len());
            if start <= s.len() && end <= s.len() && start <= end {
                return Ok(Value::String(s[start..end].to_string()));
            }
        }
    }
    Ok(Value::Null)
}

fn std_join(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        let sep = args.get(1).and_then(|v| if let Value::String(s) = v { Some(s.clone()) } else { None }).unwrap_or_default();
        let strs: Vec<String> = list.iter().map(|v| value_to_string_helper(v)).collect();
        Ok(Value::String(strs.join(&sep)))
    } else {
        Ok(Value::Null)
    }
}

fn std_contains(args: &[Value]) -> Result<Value, String> {
    match args.first() {
        Some(Value::String(s)) => {
            let sub = args.get(1).and_then(|v| if let Value::String(ss) = v { Some(ss.as_str()) } else { None });
            Ok(Value::Boolean(sub.map_or(false, |sub| s.contains(sub))))
        }
        Some(Value::List(list)) => {
            let item = args.get(1);
            Ok(Value::Boolean(item.map_or(false, |item| list.contains(item))))
        }
        _ => Ok(Value::Boolean(false)),
    }
}

fn std_replace(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(s)) = args.first() {
        let from = args.get(1).and_then(|v| if let Value::String(f) = v { Some(f.clone()) } else { None }).unwrap_or_default();
        let to = args.get(2).and_then(|v| if let Value::String(t) = v { Some(t.clone()) } else { None }).unwrap_or_default();
        Ok(Value::String(s.replace(&from, &to)))
    } else {
        Ok(Value::Null)
    }
}

fn std_trim(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::String(s)) => Value::String(s.trim().to_string()),
        _ => Value::Null,
    })
}

fn std_split(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::String(s)) = args.first() {
        let sep = args.get(1).and_then(|v| if let Value::String(ss) = v { Some(ss.clone()) } else { None }).unwrap_or_default();
        if sep.is_empty() {
            let chars: Vec<Value> = s.chars().map(|c| Value::String(c.to_string())).collect();
            Ok(Value::List(chars))
        } else {
            let parts: Vec<Value> = s.split(&sep).map(|p| Value::String(p.to_string())).collect();
            Ok(Value::List(parts))
        }
    } else {
        Ok(Value::Null)
    }
}

fn std_random(args: &[Value]) -> Result<Value, String> {
    let min = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
    let max = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(100);
    use rand::Rng;
    let val: i64 = rand::thread_rng().gen_range(min..=max);
    Ok(Value::Integer(val))
}

fn std_clamp(args: &[Value]) -> Result<Value, String> {
    let val = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
    let min = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
    let max = args.get(2).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(100.0);
    Ok(Value::Float(val.max(min).min(max)))
}

fn std_abs(args: &[Value]) -> Result<Value, String> {
    let val = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i as f64) } else if let Value::Float(f) = v { Some(*f) } else { None }).unwrap_or(0.0);
    Ok(Value::Float(val.abs()))
}

fn std_min(args: &[Value]) -> Result<Value, String> {
    let a = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
    let b = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
    Ok(Value::Integer(a.min(b)))
}

fn std_max(args: &[Value]) -> Result<Value, String> {
    let a = args.get(0).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
    let b = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i) } else { None }).unwrap_or(0);
    Ok(Value::Integer(a.max(b)))
}

fn std_round(args: &[Value]) -> Result<Value, String> {
    let val = args.get(0).and_then(|v| if let Value::Float(f) = v { Some(*f) } else if let Value::Integer(i) = v { Some(*i as f64) } else { None }).unwrap_or(0.0);
    Ok(Value::Integer(val.round() as i64))
}

fn std_floor(args: &[Value]) -> Result<Value, String> {
    let val = args.get(0).and_then(|v| if let Value::Float(f) = v { Some(*f) } else if let Value::Integer(i) = v { Some(*i as f64) } else { None }).unwrap_or(0.0);
    Ok(Value::Integer(val.floor() as i64))
}

fn std_ceil(args: &[Value]) -> Result<Value, String> {
    let val = args.get(0).and_then(|v| if let Value::Float(f) = v { Some(*f) } else if let Value::Integer(i) = v { Some(*i as f64) } else { None }).unwrap_or(0.0);
    Ok(Value::Integer(val.ceil() as i64))
}

fn std_now(_args: &[Value]) -> Result<Value, String> {
    let now = chrono::Utc::now().timestamp();
    Ok(Value::Integer(now))
}

fn std_sleep(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::Integer(ms)) = args.first() {
        std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
    }
    Ok(Value::Null)
}

fn std_push(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        let mut new_list = list.clone();
        if let Some(item) = args.get(1) {
            new_list.push(item.clone());
        }
        Ok(Value::List(new_list))
    } else {
        Ok(Value::Null)
    }
}

fn std_pop(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        let mut new_list = list.clone();
        Ok(new_list.pop().unwrap_or(Value::Null))
    } else {
        Ok(Value::Null)
    }
}

fn std_first(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::List(list)) => list.first().cloned().unwrap_or(Value::Null),
        _ => Value::Null,
    })
}

fn std_last(args: &[Value]) -> Result<Value, String> {
    Ok(match args.first() {
        Some(Value::List(list)) => list.last().cloned().unwrap_or(Value::Null),
        _ => Value::Null,
    })
}

fn std_includes(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        if let Some(item) = args.get(1) {
            return Ok(Value::Boolean(list.contains(item)));
        }
    }
    Ok(Value::Boolean(false))
}

fn std_chunk(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        let size = args.get(1).and_then(|v| if let Value::Integer(i) = v { Some(*i as usize) } else { None }).unwrap_or(1);
        let chunks: Vec<Value> = list.chunks(size).map(|c| Value::List(c.to_vec())).collect();
        Ok(Value::List(chunks))
    } else {
        Ok(Value::Null)
    }
}

fn std_filter(args: &[Value]) -> Result<Value, String> {
    if let Some(Value::List(list)) = args.first() {
        // TODO: apply function filter
        Ok(Value::List(list.clone()))
    } else {
        Ok(Value::Null)
    }
}

fn value_to_string_helper(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Integer(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Null => "null".into(),
        Value::List(l) => format!("[{}]", l.iter().map(value_to_string_helper).collect::<Vec<_>>().join(", ")),
        Value::Object(m) => format!("{{{}}}", m.iter().map(|(k, v)| format!("{}: {}", k, value_to_string_helper(v))).collect::<Vec<_>>().join(", ")),
        Value::Function(n, _) => format!("<fn {}>", n),
        Value::DiscordUser(id, _) => format!("<@{}>", id),
        Value::DiscordChannel(id, _) => format!("<#{}>", id),
        Value::DiscordRole(id, _) => format!("<@&{}>", id),
        Value::DiscordMember(id, _, _) => format!("<@{}>", id),
        Value::EmbedData(_) => "<embed>".into(),
    }
}
