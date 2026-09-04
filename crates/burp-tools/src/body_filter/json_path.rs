use pest::Parser;
use pest_derive::Parser;
use regex::Regex;
use serde_json::Value;

#[derive(Parser)]
#[grammar = "grammars/json_path.pest"]
pub struct JsonPathParser;

use Rule as JsonRule;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FilterOp {
    Eq,
    Neq,
    Lt,
    Lte,
    Gt,
    Gte,
    RegexMatch,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterVal {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonSegment {
    Field(String),
    Index(isize),
    Wildcard,
    RecursiveField(String),
    RecursiveWildcard,
    Slice {
        start: Option<isize>,
        end: Option<isize>,
        step: Option<isize>,
    },
    Union(Vec<JsonSegment>),
    Filter {
        path: Vec<String>,
        op: Option<FilterOp>,
        value: Option<FilterVal>,
    },
}

pub fn parse_json_path(path: &str) -> Result<Vec<JsonSegment>, String> {
    let pairs = JsonPathParser::parse(JsonRule::json_path, path)
        .map_err(|e| format!("JSONPath parse error: {e}"))?;

    let mut segments = Vec::new();
    for pair in pairs {
        for inner in pair.into_inner() {
            match inner.as_rule() {
                JsonRule::root_ident => {
                    segments.push(JsonSegment::Field(inner.as_str().to_string()));
                }
                JsonRule::dot_segment => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            JsonRule::wildcard => segments.push(JsonSegment::Wildcard),
                            JsonRule::ident => {
                                segments.push(JsonSegment::Field(part.as_str().to_string()))
                            }
                            _ => {}
                        }
                    }
                }
                JsonRule::recursive_segment => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            JsonRule::wildcard => segments.push(JsonSegment::RecursiveWildcard),
                            JsonRule::ident => segments
                                .push(JsonSegment::RecursiveField(part.as_str().to_string())),
                            JsonRule::bracket_segment => {
                                for b_part in part.into_inner() {
                                    match b_part.as_rule() {
                                        JsonRule::wildcard => {
                                            segments.push(JsonSegment::RecursiveWildcard)
                                        }
                                        JsonRule::string_lit | JsonRule::ident => {
                                            let s = b_part.as_str();
                                            let unquoted = unquote_str(s);
                                            segments.push(JsonSegment::RecursiveField(
                                                unquoted.to_string(),
                                            ));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                JsonRule::bracket_segment => {
                    for part in inner.into_inner() {
                        match part.as_rule() {
                            JsonRule::wildcard => segments.push(JsonSegment::Wildcard),
                            JsonRule::number => {
                                if let Ok(n) = part.as_str().parse::<isize>() {
                                    segments.push(JsonSegment::Index(n));
                                }
                            }
                            JsonRule::string_lit => {
                                let s = part.as_str();
                                let unquoted = unquote_str(s);
                                segments.push(JsonSegment::Field(unquoted.to_string()));
                            }
                            JsonRule::ident => {
                                segments.push(JsonSegment::Field(part.as_str().to_string()))
                            }
                            JsonRule::slice => {
                                let slice_str = part.as_str();
                                let parts: Vec<&str> = slice_str.split(':').collect();
                                let start =
                                    parts.first().and_then(|s| s.trim().parse::<isize>().ok());
                                let end = parts.get(1).and_then(|s| s.trim().parse::<isize>().ok());
                                let step =
                                    parts.get(2).and_then(|s| s.trim().parse::<isize>().ok());
                                segments.push(JsonSegment::Slice { start, end, step });
                            }
                            JsonRule::union_segment => {
                                let mut union_items = Vec::new();
                                for u_part in part.into_inner() {
                                    if u_part.as_rule() == JsonRule::union_item {
                                        for item in u_part.into_inner() {
                                            match item.as_rule() {
                                                JsonRule::number => {
                                                    if let Ok(n) = item.as_str().parse::<isize>() {
                                                        union_items.push(JsonSegment::Index(n));
                                                    }
                                                }
                                                JsonRule::string_lit => {
                                                    let s = item.as_str();
                                                    union_items.push(JsonSegment::Field(
                                                        unquote_str(s).to_string(),
                                                    ));
                                                }
                                                JsonRule::ident => {
                                                    union_items.push(JsonSegment::Field(
                                                        item.as_str().to_string(),
                                                    ));
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                if !union_items.is_empty() {
                                    segments.push(JsonSegment::Union(union_items));
                                }
                            }
                            JsonRule::filter_expr => {
                                for f_cond in part.into_inner() {
                                    if f_cond.as_rule() == JsonRule::filter_condition {
                                        let mut filter_path = Vec::new();
                                        let mut filter_op = None;
                                        let mut filter_val = None;

                                        for cond_part in f_cond.into_inner() {
                                            match cond_part.as_rule() {
                                                JsonRule::filter_path => {
                                                    let raw = cond_part
                                                        .as_str()
                                                        .trim_start_matches('@')
                                                        .trim_start_matches('$')
                                                        .trim_start_matches('.');
                                                    for ident in cond_part.into_inner() {
                                                        if ident.as_rule() == JsonRule::ident {
                                                            filter_path
                                                                .push(ident.as_str().to_string());
                                                        }
                                                    }
                                                    if filter_path.is_empty() {
                                                        for piece in raw.split('.') {
                                                            if !piece.is_empty() {
                                                                filter_path.push(piece.to_string());
                                                            }
                                                        }
                                                    }
                                                }
                                                JsonRule::filter_op => {
                                                    filter_op = match cond_part.as_str() {
                                                        "==" => Some(FilterOp::Eq),
                                                        "!=" => Some(FilterOp::Neq),
                                                        "<=" => Some(FilterOp::Lte),
                                                        ">=" => Some(FilterOp::Gte),
                                                        "<" => Some(FilterOp::Lt),
                                                        ">" => Some(FilterOp::Gt),
                                                        "=~" => Some(FilterOp::RegexMatch),
                                                        _ => None,
                                                    };
                                                }
                                                JsonRule::filter_val => {
                                                    for val_item in cond_part.into_inner() {
                                                        filter_val = match val_item.as_rule() {
                                                            JsonRule::string_lit => {
                                                                Some(FilterVal::String(
                                                                    unquote_str(val_item.as_str())
                                                                        .to_string(),
                                                                ))
                                                            }
                                                            JsonRule::number => val_item
                                                                .as_str()
                                                                .parse::<f64>()
                                                                .ok()
                                                                .map(FilterVal::Number),
                                                            JsonRule::boolean => {
                                                                Some(FilterVal::Bool(
                                                                    val_item.as_str() == "true",
                                                                ))
                                                            }
                                                            _ => {
                                                                if val_item.as_str() == "null" {
                                                                    Some(FilterVal::Null)
                                                                } else {
                                                                    None
                                                                }
                                                            }
                                                        };
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        segments.push(JsonSegment::Filter {
                                            path: filter_path,
                                            op: filter_op,
                                            value: filter_val,
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                JsonRule::EOI => {}
                _ => {}
            }
        }
    }
    Ok(segments)
}

fn unquote_str(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 { &s[1..s.len() - 1] } else { s }
    } else {
        s
    }
}

pub fn extract_json_path(json_text: &str, path: &str) -> Result<Vec<Value>, String> {
    let parsed: Value =
        serde_json::from_str(json_text).map_err(|e| format!("Invalid JSON for extraction: {e}"))?;

    let segments = parse_json_path(path)?;
    let mut current = vec![parsed];

    for segment in &segments {
        let mut next = Vec::new();
        for val in &current {
            match segment {
                JsonSegment::Field(field) => {
                    if let Some(v) = val.get(field) {
                        next.push(v.clone());
                    }
                }
                JsonSegment::Index(idx) => {
                    if let Value::Array(arr) = val {
                        let actual_idx = if *idx < 0 {
                            arr.len() as isize + *idx
                        } else {
                            *idx
                        };
                        if actual_idx >= 0 && (actual_idx as usize) < arr.len() {
                            next.push(arr[actual_idx as usize].clone());
                        }
                    }
                }
                JsonSegment::Wildcard => {
                    if let Value::Array(arr) = val {
                        next.extend(arr.iter().cloned());
                    } else if let Value::Object(map) = val {
                        next.extend(map.values().cloned());
                    }
                }
                JsonSegment::RecursiveField(field) => {
                    recursive_collect_field(val, field, &mut next);
                }
                JsonSegment::RecursiveWildcard => {
                    recursive_collect_all(val, &mut next);
                }
                JsonSegment::Slice { start, end, step } => {
                    if let Value::Array(arr) = val {
                        let len = arr.len() as isize;
                        let step_val = step.unwrap_or(1);
                        if step_val > 0 {
                            let s = start
                                .map(|v| if v < 0 { (len + v).max(0) } else { v.min(len) })
                                .unwrap_or(0) as usize;
                            let e = end
                                .map(|v| if v < 0 { (len + v).max(0) } else { v.min(len) })
                                .unwrap_or(len) as usize;
                            let mut i = s;
                            while i < e && i < arr.len() {
                                next.push(arr[i].clone());
                                i += step_val as usize;
                            }
                        }
                    }
                }
                JsonSegment::Union(items) => {
                    for item in items {
                        match item {
                            JsonSegment::Field(f) => {
                                if let Some(v) = val.get(f) {
                                    next.push(v.clone());
                                }
                            }
                            JsonSegment::Index(idx) => {
                                if let Value::Array(arr) = val {
                                    let actual_idx = if *idx < 0 {
                                        arr.len() as isize + *idx
                                    } else {
                                        *idx
                                    };
                                    if actual_idx >= 0 && (actual_idx as usize) < arr.len() {
                                        next.push(arr[actual_idx as usize].clone());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                JsonSegment::Filter {
                    path: filter_path,
                    op,
                    value: expected_val,
                } => {
                    if let Value::Array(arr) = val {
                        for item in arr {
                            if evaluate_json_filter(item, filter_path, op, expected_val) {
                                next.push(item.clone());
                            }
                        }
                    } else if evaluate_json_filter(val, filter_path, op, expected_val) {
                        next.push(val.clone());
                    }
                }
            }
        }
        current = next;
    }

    Ok(current)
}

fn recursive_collect_field(val: &Value, target: &str, out: &mut Vec<Value>) {
    match val {
        Value::Object(map) => {
            if let Some(v) = map.get(target) {
                out.push(v.clone());
            }
            for child in map.values() {
                recursive_collect_field(child, target, out);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                recursive_collect_field(item, target, out);
            }
        }
        _ => {}
    }
}

fn recursive_collect_all(val: &Value, out: &mut Vec<Value>) {
    match val {
        Value::Object(map) => {
            for v in map.values() {
                out.push(v.clone());
                recursive_collect_all(v, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.push(v.clone());
                recursive_collect_all(v, out);
            }
        }
        _ => {}
    }
}

fn evaluate_json_filter(
    val: &Value,
    path: &[String],
    op: &Option<FilterOp>,
    expected: &Option<FilterVal>,
) -> bool {
    let mut current = val;
    for segment in path {
        match current {
            Value::Object(map) => {
                if let Some(v) = map.get(segment) {
                    current = v;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
    }

    match (op, expected) {
        (None, _) => match current {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Object(m) => !m.is_empty(),
            Value::Number(_) => true,
        },
        (Some(FilterOp::Eq), Some(exp)) => match (current, exp) {
            (Value::String(s), FilterVal::String(e)) => s == e,
            (Value::Number(n), FilterVal::Number(e)) => {
                n.as_f64().is_some_and(|v| (v - e).abs() < f64::EPSILON)
            }
            (Value::Bool(b), FilterVal::Bool(e)) => b == e,
            (Value::Null, FilterVal::Null) => true,
            _ => false,
        },
        (Some(FilterOp::Neq), Some(exp)) => match (current, exp) {
            (Value::String(s), FilterVal::String(e)) => s != e,
            (Value::Number(n), FilterVal::Number(e)) => {
                n.as_f64().is_none_or(|v| (v - e).abs() >= f64::EPSILON)
            }
            (Value::Bool(b), FilterVal::Bool(e)) => b != e,
            (Value::Null, FilterVal::Null) => false,
            _ => true,
        },
        (Some(FilterOp::Lt), Some(FilterVal::Number(e))) => {
            current.as_f64().is_some_and(|v| v < *e)
        }
        (Some(FilterOp::Lte), Some(FilterVal::Number(e))) => {
            current.as_f64().is_some_and(|v| v <= *e)
        }
        (Some(FilterOp::Gt), Some(FilterVal::Number(e))) => {
            current.as_f64().is_some_and(|v| v > *e)
        }
        (Some(FilterOp::Gte), Some(FilterVal::Number(e))) => {
            current.as_f64().is_some_and(|v| v >= *e)
        }
        (Some(FilterOp::RegexMatch), Some(FilterVal::String(pat))) => {
            if let Value::String(s) = current {
                Regex::new(pat).is_ok_and(|re| re.is_match(s))
            } else {
                false
            }
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pest_json_path() {
        let json = r#"{
            "data": {
                "users": [
                    {"id": 1, "name": "Alice", "role": "admin", "active": true},
                    {"id": 2, "name": "Bob", "role": "user", "active": false}
                ],
                "meta": {
                    "total": 2
                }
            }
        }"#;

        let res = extract_json_path(json, "$.data.users[*].name").unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "Alice");
        assert_eq!(res[1], "Bob");

        let res_single = extract_json_path(json, "data.meta.total").unwrap();
        assert_eq!(res_single.len(), 1);
        assert_eq!(res_single[0], 2);

        let res_idx = extract_json_path(json, "$.data.users[0].id").unwrap();
        assert_eq!(res_idx.len(), 1);
        assert_eq!(res_idx[0], 1);

        // Recursive descent
        let res_rec = extract_json_path(json, "$..name").unwrap();
        assert_eq!(res_rec.len(), 2);
        assert_eq!(res_rec[0], "Alice");
        assert_eq!(res_rec[1], "Bob");

        // Filter expression
        let res_filter =
            extract_json_path(json, "$.data.users[?(@.role == 'admin')].name").unwrap();
        assert_eq!(res_filter.len(), 1);
        assert_eq!(res_filter[0], "Alice");

        // Slice expression
        let res_slice = extract_json_path(json, "$.data.users[0:1].name").unwrap();
        assert_eq!(res_slice.len(), 1);
        assert_eq!(res_slice[0], "Alice");

        // Union expression
        let res_union = extract_json_path(json, "$.data.users[0]['id', 'name']").unwrap();
        assert_eq!(res_union.len(), 2);
        assert_eq!(res_union[0], 1);
        assert_eq!(res_union[1], "Alice");
    }
}
