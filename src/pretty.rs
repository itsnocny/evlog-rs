use crate::event::{Event, ErrorInfo};
use crate::level::Level;
use serde_json::Value;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[90m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";

fn colors() -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
    if std::env::var_os("NO_COLOR").is_some() {
        ("", "", "", "", "", "", "", "")
    } else {
        (RESET, BOLD, DIM, RED, YELLOW, GREEN, CYAN, MAGENTA)
    }
}

pub fn format_event(event: &Event) -> String {
    let mut out = String::new();
    let (reset, bold, dim, red, yellow, green, cyan, magenta) = colors();

    let level_color = match event.level {
        Level::Error => red,
        Level::Warn => yellow,
        Level::Info => green,
        Level::Debug => cyan,
    };

    let ts = event.timestamp.format("%H:%M:%S%.3f").to_string();

    let is_simple = event.fields.is_empty() && event.error.is_none() 
        && event.method.is_none() && event.path.is_none() && event.status.is_none();

    if is_simple {
        out.push_str(&format!("{}{}{}", dim, ts, reset));
        if event.level != Level::Info {
            out.push_str(&format!(" {}{}{}", level_color, event.level.to_string().to_uppercase(), reset));
        }
        if let Some(ref tag) = event.tag {
            out.push_str(&format!(" {}{}[{}]{}", bold, cyan, tag, reset));
        }
        if let Some(ref msg) = event.message {
            out.push_str(&format!(" {}", msg));
        }
        return out;
    }

    if event.level != Level::Info {
        out.push_str(&format!("{}{}{}", level_color, event.level.to_string().to_uppercase(), reset));
    } else {
        out.push_str(&format!("{}[{}]{}", level_color, event.level.to_string().to_uppercase(), reset));
    }

    if let Some(ref tag) = event.tag {
        out.push_str(&format!(" {}{}[{}]{}", bold, cyan, tag, reset));
    }

    if let Some(ref method) = event.method {
        out.push_str(&format!(" {}", method));
    }
    if let Some(ref path) = event.path {
        out.push_str(&format!(" {}", path));
    }
    if let Some(status) = event.status {
        let status_color = if status >= 500 { red } else if status >= 400 { yellow } else { green };
        out.push_str(&format!(" {}{}{}", status_color, status, reset));
    }
    if let Some(dur) = event.duration_ms {
        out.push_str(&format!(" {}in {}ms{}", dim, dur, reset));
    }
    if event.method.is_none() && event.path.is_none() {
        if let Some(ref msg) = event.message {
            out.push_str(&format!(" {}", msg));
        }
    }

    out.push('\n');

    let mut children = Vec::new();
    if let Some(ref err) = event.error {
        children.push(("error".to_string(), format_error(err, dim, reset)));
    }
    for (k, v) in &event.fields {
        children.push((k.clone(), format_value_inline_inner(v, magenta, reset)));
    }

    let len = children.len();
    for (i, (k, v)) in children.iter().enumerate() {
        let prefix = if i == len - 1 { "└─" } else { "├─" };
        let mut lines = v.lines();
        if let Some(first) = lines.next() {
            out.push_str(&format!("  {} {}{}{}: {}\n", prefix, magenta, k, reset, first));
        }
        for line in lines {
            let space_prefix = if i == len - 1 { " " } else { "│" };
            out.push_str(&format!("  {}    {}\n", space_prefix, line));
        }
    }

    out.trim_end().to_string()
}

fn format_error(err: &ErrorInfo, dim: &str, reset: &str) -> String {
    let mut out = err.message.clone();
    
    if let Some(ref internal) = err.internal {
        if let Some(loc) = internal.get("location").and_then(|v| v.as_str()) {
            out.push_str(&format!("\n{}At:  {}{}", dim, reset, loc));
        }
    }
    
    if let Some(ref why) = err.why {
        out.push_str(&format!("\n{}Why: {}{}", dim, reset, why));
    }
    if let Some(ref fix) = err.fix {
        out.push_str(&format!("\n{}Fix: {}{}", dim, reset, fix));
    }
    if let Some(ref stack) = err.stack {
        if !stack.is_empty() && stack != "disabled backtrace" {
            out.push_str(&format!("\n{}Stacktrace:\n{}{}", dim, stack, reset));
        }
    }
    out
}

pub fn format_value_inline(value: &Value) -> String {
    let (_, _, _, _, _, _, _, magenta) = colors();
    let (reset, _, _, _, _, _, _, _) = colors();
    format_value_inline_inner(value, magenta, reset)
}

fn format_value_inline_inner(value: &Value, magenta: &str, reset: &str) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(a) => {
            let items: Vec<String> = a.iter().map(|v| format_value_inline_inner(v, magenta, reset)).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Object(o) => {
            let items: Vec<String> = o.iter().map(|(k, v)| {
                format!("{}{}={}{}", magenta, k, reset, format_value_inline_inner(v, magenta, reset))
            }).collect();
            format!("{{ {} }}", items.join(" "))
        }
    }
}
