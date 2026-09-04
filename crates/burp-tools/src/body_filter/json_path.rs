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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FilterPathPrefix {
    Current,
    Root,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FilterPath {
    pub prefix: FilterPathPrefix,
    pub segments: Vec<JsonSegment>,
}

#[derive(Debug, Clone)]
pub enum FilterComparable {
    Literal(FilterVal),
    Path(FilterPath),
    Function {
        name: String,
        args: Vec<FilterComparable>,
        compiled_regex: Option<Regex>,
    },
}
#[derive(Debug, Clone, PartialEq)]
enum ResolvedComparable {
    Missing,
    Value(Value),
}

impl PartialEq for FilterComparable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Literal(a), Self::Literal(b)) => a == b,
            (Self::Path(a), Self::Path(b)) => a == b,
            (
                Self::Function {
                    name: n1, args: a1, ..
                },
                Self::Function {
                    name: n2, args: a2, ..
                },
            ) => n1 == n2 && a1 == a2,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FilterExpr {
    Test(FilterComparable),
    Comparison {
        left: FilterComparable,
        op: FilterOp,
        right: FilterComparable,
    },
    Not(Box<FilterExpr>),
    And(Vec<FilterExpr>),
    Or(Vec<FilterExpr>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum JsonSegment {
    Field(String),
    Index(isize),
    Wildcard,
    RecursiveField(String),
    RecursiveWildcard,
    Recursive(Box<JsonSegment>),
    Slice {
        start: Option<isize>,
        end: Option<isize>,
        step: Option<isize>,
    },
    Union(Vec<JsonSegment>),
    Filter(FilterExpr),
}

pub fn unescape_json_string(s: &str) -> Result<String, String> {
    let trimmed = s.trim();
    let content = if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    let mut out = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\'') => out.push('\''),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('b') => out.push('\x08'),
                Some('f') => out.push('\x0C'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(hc) = chars.next() {
                            if hc.is_ascii_hexdigit() {
                                hex.push(hc);
                            } else {
                                return Err(format!("Invalid unicode hex digit: {hc}"));
                            }
                        } else {
                            return Err("Incomplete \\u hex escape".to_string());
                        }
                    }
                    let cp = u32::from_str_radix(&hex, 16)
                        .map_err(|e| format!("Invalid hex escape: {e}"))?;

                    // Check for UTF-16 surrogate pair
                    if (0xD800..=0xDBFF).contains(&cp) {
                        let mut lookahead = chars.clone();
                        if lookahead.next() == Some('\\') && lookahead.next() == Some('u') {
                            let mut low_hex = String::with_capacity(4);
                            let mut valid_low = true;
                            for _ in 0..4 {
                                if let Some(lhc) = lookahead.next() {
                                    if lhc.is_ascii_hexdigit() {
                                        low_hex.push(lhc);
                                    } else {
                                        valid_low = false;
                                        break;
                                    }
                                } else {
                                    valid_low = false;
                                    break;
                                }
                            }
                            if valid_low
                                && low_hex.len() == 4
                                && let Ok(low_cp) = u32::from_str_radix(&low_hex, 16)
                                && (0xDC00..=0xDFFF).contains(&low_cp)
                            {
                                chars.next(); // '\'
                                chars.next(); // 'u'
                                for _ in 0..4 {
                                    chars.next();
                                }
                                let full_cp = 0x10000 + ((cp - 0xD800) << 10) + (low_cp - 0xDC00);
                                if let Some(ch) = char::from_u32(full_cp) {
                                    out.push(ch);
                                    continue;
                                }
                            }
                        }
                    }

                    if let Some(ch) = char::from_u32(cp) {
                        out.push(ch);
                    } else {
                        out.push('\u{FFFD}');
                    }
                }
                Some(other) => {
                    out.push(other);
                }
                None => return Err("Dangling backslash in string escape".to_string()),
            }
        } else {
            out.push(c);
        }
    }

    Ok(out)
}

pub fn parse_json_path(path: &str) -> Result<Vec<JsonSegment>, String> {
    let pairs = JsonPathParser::parse(JsonRule::json_path, path)
        .map_err(|e| format!("JSONPath parse error: {e}"))?;

    let mut segments = Vec::new();

    for pair in pairs {
        if pair.as_rule() == JsonRule::json_path {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    JsonRule::root_scope => {}
                    JsonRule::dot_segment => {
                        for sub in inner.into_inner() {
                            match sub.as_rule() {
                                JsonRule::ident => {
                                    segments.push(JsonSegment::Field(sub.as_str().to_string()));
                                }
                                JsonRule::wildcard => {
                                    segments.push(JsonSegment::Wildcard);
                                }
                                _ => {}
                            }
                        }
                    }
                    JsonRule::root_ident => {
                        segments.push(JsonSegment::Field(inner.as_str().to_string()));
                    }
                    JsonRule::recursive_segment => {
                        for sub in inner.into_inner() {
                            match sub.as_rule() {
                                JsonRule::ident => {
                                    segments.push(JsonSegment::RecursiveField(
                                        sub.as_str().to_string(),
                                    ));
                                }
                                JsonRule::wildcard => {
                                    segments.push(JsonSegment::RecursiveWildcard);
                                }
                                JsonRule::bracket_segment => {
                                    let bracket_seg = parse_bracket_segment(sub)?;
                                    segments.push(JsonSegment::Recursive(Box::new(bracket_seg)));
                                }
                                _ => {}
                            }
                        }
                    }
                    JsonRule::bracket_segment => {
                        let seg = parse_bracket_segment(inner)?;
                        segments.push(seg);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(segments)
}

fn parse_bracket_segment(pair: pest::iterators::Pair<JsonRule>) -> Result<JsonSegment, String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::union_segment => {
                let mut items = Vec::new();
                for item_pair in inner.into_inner() {
                    if item_pair.as_rule() == JsonRule::selector_item {
                        let seg = parse_selector_item(item_pair)?;
                        items.push(seg);
                    }
                }
                return Ok(JsonSegment::Union(items));
            }
            JsonRule::selector_item => {
                return parse_selector_item(inner);
            }
            _ => {}
        }
    }
    Err("Empty bracket segment".to_string())
}

fn parse_selector_item(pair: pest::iterators::Pair<JsonRule>) -> Result<JsonSegment, String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::filter_expr => {
                let expr = parse_filter_expr(inner)?;
                return Ok(JsonSegment::Filter(expr));
            }
            JsonRule::slice => {
                return parse_slice(inner);
            }
            JsonRule::wildcard => {
                return Ok(JsonSegment::Wildcard);
            }
            JsonRule::number => {
                let num_str = inner.as_str().trim();
                let idx = num_str
                    .parse::<isize>()
                    .map_err(|e| format!("Invalid index: {e}"))?;
                return Ok(JsonSegment::Index(idx));
            }
            JsonRule::string_lit => {
                let unesc = unescape_json_string(inner.as_str())?;
                return Ok(JsonSegment::Field(unesc));
            }
            JsonRule::ident => {
                return Ok(JsonSegment::Field(inner.as_str().to_string()));
            }
            _ => {}
        }
    }
    Err("Unknown selector item".to_string())
}

fn parse_slice(pair: pest::iterators::Pair<JsonRule>) -> Result<JsonSegment, String> {
    let raw = pair.as_str().trim();
    let parts: Vec<&str> = raw.split(':').collect();

    let parse_num = |s: &str| -> Result<Option<isize>, String> {
        let s = s.trim();
        if s.is_empty() {
            Ok(None)
        } else {
            let n = s
                .parse::<isize>()
                .map_err(|e| format!("Invalid slice bound: {e}"))?;
            Ok(Some(n))
        }
    };

    let start = if !parts.is_empty() {
        parse_num(parts[0])?
    } else {
        None
    };

    let end = if parts.len() > 1 {
        parse_num(parts[1])?
    } else {
        None
    };

    let step = if parts.len() > 2 {
        parse_num(parts[2])?
    } else {
        None
    };

    Ok(JsonSegment::Slice { start, end, step })
}

fn parse_filter_expr(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterExpr, String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::logical_or_expr => {
                return parse_logical_or_expr(inner);
            }
            JsonRule::paren_logical_expr => {
                for p_inner in inner.into_inner() {
                    if p_inner.as_rule() == JsonRule::logical_or_expr {
                        return parse_logical_or_expr(p_inner);
                    }
                }
            }
            _ => {}
        }
    }
    Err("Invalid filter expr".to_string())
}

fn parse_logical_or_expr(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterExpr, String> {
    let mut and_exprs = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == JsonRule::logical_and_expr {
            let and_expr = parse_logical_and_expr(inner)?;
            and_exprs.push(and_expr);
        }
    }
    if and_exprs.len() == 1 {
        Ok(and_exprs.remove(0))
    } else {
        Ok(FilterExpr::Or(and_exprs))
    }
}

fn parse_logical_and_expr(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterExpr, String> {
    let mut unaries = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == JsonRule::unary_expr {
            let unary = parse_unary_expr(inner)?;
            unaries.push(unary);
        }
    }
    if unaries.len() == 1 {
        Ok(unaries.remove(0))
    } else {
        Ok(FilterExpr::And(unaries))
    }
}

fn parse_unary_expr(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterExpr, String> {
    let mut is_not = false;
    let mut primary = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::logical_not => {
                is_not = true;
            }
            JsonRule::primary_filter_expr => {
                primary = Some(parse_primary_filter_expr(inner)?);
            }
            _ => {}
        }
    }

    let expr = primary.ok_or_else(|| "Missing primary expr in unary".to_string())?;
    if is_not {
        Ok(FilterExpr::Not(Box::new(expr)))
    } else {
        Ok(expr)
    }
}

fn parse_primary_filter_expr(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterExpr, String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::paren_logical_expr => {
                for sub in inner.into_inner() {
                    if sub.as_rule() == JsonRule::logical_or_expr {
                        return parse_logical_or_expr(sub);
                    }
                }
            }
            JsonRule::comparison_or_test_expr => {
                return parse_comparison_or_test_expr(inner);
            }
            _ => {}
        }
    }
    Err("Invalid primary filter expr".to_string())
}

fn parse_comparison_or_test_expr(
    pair: pest::iterators::Pair<JsonRule>,
) -> Result<FilterExpr, String> {
    let mut left = None;
    let mut op = None;
    let mut right = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::comparable_val => {
                let comp = parse_comparable_val(inner)?;
                if left.is_none() {
                    left = Some(comp);
                } else {
                    right = Some(comp);
                }
            }
            JsonRule::filter_op => {
                let op_val = match inner.as_str() {
                    "==" => FilterOp::Eq,
                    "!=" => FilterOp::Neq,
                    "<=" => FilterOp::Lte,
                    ">=" => FilterOp::Gte,
                    "<" => FilterOp::Lt,
                    ">" => FilterOp::Gt,
                    "=~" => FilterOp::RegexMatch,
                    other => return Err(format!("Unknown filter operator: {other}")),
                };
                op = Some(op_val);
            }
            _ => {}
        }
    }

    let left = left.ok_or_else(|| "Missing left comparable".to_string())?;
    if let (Some(op), Some(right)) = (op, right) {
        Ok(FilterExpr::Comparison { left, op, right })
    } else {
        Ok(FilterExpr::Test(left))
    }
}

fn parse_comparable_val(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterComparable, String> {
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::filter_literal => {
                return parse_filter_literal(inner);
            }
            JsonRule::filter_path => {
                return parse_filter_path(inner);
            }
            JsonRule::func_call => {
                return parse_func_call(inner);
            }
            _ => {}
        }
    }
    Err("Invalid comparable val".to_string())
}

fn parse_filter_literal(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterComparable, String> {
    let raw_str = pair.as_str().trim().to_string();
    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::string_lit => {
                let unesc = unescape_json_string(inner.as_str())?;
                return Ok(FilterComparable::Literal(FilterVal::String(unesc)));
            }
            JsonRule::number => {
                let num = inner
                    .as_str()
                    .trim()
                    .parse::<f64>()
                    .map_err(|e| format!("Invalid float: {e}"))?;
                return Ok(FilterComparable::Literal(FilterVal::Number(num)));
            }
            JsonRule::boolean => {
                let b = inner.as_str().trim() == "true";
                return Ok(FilterComparable::Literal(FilterVal::Bool(b)));
            }
            _ => {
                if inner.as_str().trim() == "null" {
                    return Ok(FilterComparable::Literal(FilterVal::Null));
                }
            }
        }
    }
    if raw_str == "null" {
        Ok(FilterComparable::Literal(FilterVal::Null))
    } else {
        Err(format!("Unknown literal: {raw_str}"))
    }
}

fn parse_filter_path(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterComparable, String> {
    let raw = pair.as_str().trim();
    let (prefix, rest) = if let Some(stripped) = raw.strip_prefix('$') {
        (FilterPathPrefix::Root, stripped)
    } else if let Some(stripped) = raw.strip_prefix('@') {
        (FilterPathPrefix::Current, stripped)
    } else {
        (FilterPathPrefix::Current, raw)
    };

    let segments = if rest.is_empty() {
        Vec::new()
    } else {
        let normalized = if rest.starts_with('.') || rest.starts_with('[') {
            format!("${rest}")
        } else {
            format!("$.{rest}")
        };
        parse_json_path(&normalized)?
    };

    Ok(FilterComparable::Path(FilterPath { prefix, segments }))
}

fn parse_func_call(pair: pest::iterators::Pair<JsonRule>) -> Result<FilterComparable, String> {
    let mut func_name = String::new();
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            JsonRule::func_name => {
                func_name = inner.as_str().to_string();
            }
            JsonRule::func_arg => {
                for sub in inner.into_inner() {
                    if sub.as_rule() == JsonRule::comparable_val {
                        let comp = parse_comparable_val(sub)?;
                        args.push(comp);
                    }
                }
            }
            _ => {}
        }
    }

    match func_name.as_str() {
        "match" | "search" => {
            if args.len() != 2 {
                return Err(format!("{func_name}() requires exactly 2 arguments"));
            }

            let pattern_str = match &args[1] {
                FilterComparable::Literal(FilterVal::String(s)) => s.clone(),
                _ => {
                    return Err(format!(
                        "{func_name}() second argument must be a string pattern"
                    ));
                }
            };

            let regex_str = if func_name == "match" {
                format!("^(?:{pattern_str})$")
            } else {
                pattern_str
            };

            let compiled = Regex::new(&regex_str)
                .map_err(|e| format!("Invalid regex in {func_name}(): {e}"))?;

            Ok(FilterComparable::Function {
                name: func_name,
                args,
                compiled_regex: Some(compiled),
            })
        }
        unsupported => Err(format!("Unsupported function: {unsupported}")),
    }
}

// ---------------- JSONPath Evaluator ----------------

pub fn extract_json_path(json_str: &str, path_str: &str) -> Result<Vec<Value>, String> {
    const MAX_INPUT_SIZE: usize = 20_000_000;
    if json_str.len() > MAX_INPUT_SIZE {
        return Err("JSON input exceeds size limit".to_string());
    }

    let root_val: Value =
        serde_json::from_str(json_str).map_err(|e| format!("JSON parse error: {e}"))?;

    let segments = parse_json_path(path_str)?;

    let mut current = vec![&root_val];
    for seg in &segments {
        let mut next = Vec::new();
        for val in current {
            apply_segment(val, &root_val, seg, &mut next, 0)?;
        }
        current = next;
    }

    let results: Vec<Value> = current.into_iter().cloned().collect();

    Ok(results)
}

fn apply_segment<'a>(
    val: &'a Value,
    root: &'a Value,
    seg: &JsonSegment,
    out: &mut Vec<&'a Value>,
    depth: usize,
) -> Result<(), String> {
    const MAX_DEPTH: usize = 512;
    if depth > MAX_DEPTH {
        return Err("Maximum JSONPath recursion depth exceeded".to_string());
    }

    match seg {
        JsonSegment::Field(name) => {
            if let Value::Object(map) = val
                && let Some(v) = map.get(name)
            {
                out.push(v);
            }
        }
        JsonSegment::Index(idx) => {
            if let Value::Array(arr) = val {
                let actual_idx = if *idx < 0 {
                    (arr.len() as isize) + *idx
                } else {
                    *idx
                };
                if actual_idx >= 0 && (actual_idx as usize) < arr.len() {
                    out.push(&arr[actual_idx as usize]);
                }
            }
        }
        JsonSegment::Wildcard => match val {
            Value::Object(map) => {
                for v in map.values() {
                    out.push(v);
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    out.push(v);
                }
            }
            _ => {}
        },
        JsonSegment::RecursiveField(name) => {
            collect_recursive_field(val, name, out, depth);
        }
        JsonSegment::RecursiveWildcard => {
            collect_recursive_wildcard(val, out, depth);
        }
        JsonSegment::Recursive(sub_seg) => {
            collect_recursive_selector(val, root, sub_seg, out, depth)?;
        }
        JsonSegment::Slice { start, end, step } => {
            if let Value::Array(arr) = val {
                let sliced = slice_array(arr, *start, *end, *step);
                for v in sliced {
                    out.push(v);
                }
            }
        }
        JsonSegment::Union(items) => {
            for item in items {
                apply_segment(val, root, item, out, depth + 1)?;
            }
        }
        JsonSegment::Filter(expr) => {
            if let Value::Array(arr) = val {
                for elem in arr {
                    if evaluate_filter_expr(expr, elem, root)? {
                        out.push(elem);
                    }
                }
            } else if let Value::Object(map) = val {
                for elem in map.values() {
                    if evaluate_filter_expr(expr, elem, root)? {
                        out.push(elem);
                    }
                }
            }
        }
    }

    Ok(())
}

fn collect_recursive_field<'a>(val: &'a Value, name: &str, out: &mut Vec<&'a Value>, depth: usize) {
    if depth > 512 {
        return;
    }
    match val {
        Value::Object(map) => {
            if let Some(v) = map.get(name) {
                out.push(v);
            }
            for v in map.values() {
                collect_recursive_field(v, name, out, depth + 1);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_recursive_field(v, name, out, depth + 1);
            }
        }
        _ => {}
    }
}

fn collect_recursive_wildcard<'a>(val: &'a Value, out: &mut Vec<&'a Value>, depth: usize) {
    if depth > 512 {
        return;
    }
    match val {
        Value::Object(map) => {
            for v in map.values() {
                out.push(v);
                collect_recursive_wildcard(v, out, depth + 1);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.push(v);
                collect_recursive_wildcard(v, out, depth + 1);
            }
        }
        _ => {}
    }
}

fn collect_recursive_selector<'a>(
    val: &'a Value,
    root: &'a Value,
    seg: &JsonSegment,
    out: &mut Vec<&'a Value>,
    depth: usize,
) -> Result<(), String> {
    if depth > 512 {
        return Ok(());
    }

    apply_segment(val, root, seg, out, depth + 1)?;

    match val {
        Value::Object(map) => {
            for v in map.values() {
                collect_recursive_selector(v, root, seg, out, depth + 1)?;
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_recursive_selector(v, root, seg, out, depth + 1)?;
            }
        }
        _ => {}
    }

    Ok(())
}

fn slice_array(
    arr: &[Value],
    start: Option<isize>,
    end: Option<isize>,
    step: Option<isize>,
) -> Vec<&Value> {
    let step = step.unwrap_or(1);
    if step == 0 {
        return Vec::new();
    }

    let len = arr.len() as isize;
    if len == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();

    if step > 0 {
        let s = match start {
            Some(i) if i < 0 => (len + i).max(0),
            Some(i) => i.clamp(0, len),
            None => 0,
        };
        let e = match end {
            Some(i) if i < 0 => (len + i).max(0),
            Some(i) => i.clamp(0, len),
            None => len,
        };

        let mut i = s;
        while i < e {
            if i >= 0 && (i as usize) < arr.len() {
                result.push(&arr[i as usize]);
            }
            i += step;
        }
    } else {
        let s = match start {
            Some(i) if i < 0 => (len + i).max(-1),
            Some(i) => i.min(len - 1),
            None => len - 1,
        };
        let e = match end {
            Some(i) if i < 0 => (len + i).max(-1),
            Some(i) => i.min(len - 1),
            None => -1,
        };

        let mut i = s;
        while i > e {
            if i >= 0 && (i as usize) < arr.len() {
                result.push(&arr[i as usize]);
            }
            i += step;
        }
    }

    result
}

fn evaluate_filter_expr(expr: &FilterExpr, current: &Value, root: &Value) -> Result<bool, String> {
    match expr {
        FilterExpr::Test(comp) => match comp {
            FilterComparable::Path(path) => Ok(!resolve_path(path, current, root)?.is_empty()),
            _ => Ok(match resolve_comparable(comp, current, root)? {
                ResolvedComparable::Missing => false,
                ResolvedComparable::Value(value) => is_truthy(&value),
            }),
        },
        FilterExpr::Comparison { left, op, right } => {
            let left = resolve_comparable(left, current, root)?;
            let right = resolve_comparable(right, current, root)?;
            Ok(compare_resolved_values(&left, op, &right))
        }
        FilterExpr::Not(inner) => Ok(!evaluate_filter_expr(inner, current, root)?),
        FilterExpr::And(inners) => {
            for inner in inners {
                if !evaluate_filter_expr(inner, current, root)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        FilterExpr::Or(inners) => {
            for inner in inners {
                if evaluate_filter_expr(inner, current, root)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

fn resolve_path<'a>(
    path: &FilterPath,
    current: &'a Value,
    root: &'a Value,
) -> Result<Vec<&'a Value>, String> {
    let base = match path.prefix {
        FilterPathPrefix::Current => current,
        FilterPathPrefix::Root => root,
    };
    let mut current_nodes = vec![base];
    for segment in &path.segments {
        let mut next_nodes = Vec::new();
        for node in current_nodes {
            apply_segment(node, root, segment, &mut next_nodes, 0)?;
        }
        current_nodes = next_nodes;
    }
    Ok(current_nodes)
}

fn resolve_comparable(
    comp: &FilterComparable,
    current: &Value,
    root: &Value,
) -> Result<ResolvedComparable, String> {
    match comp {
        FilterComparable::Literal(literal) => Ok(ResolvedComparable::Value(match literal {
            FilterVal::String(value) => Value::String(value.clone()),
            FilterVal::Number(value) => serde_json::Number::from_f64(*value)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            FilterVal::Bool(value) => Value::Bool(*value),
            FilterVal::Null => Value::Null,
        })),
        FilterComparable::Path(path) => {
            let nodes = resolve_path(path, current, root)?;
            Ok(match nodes.as_slice() {
                [] => ResolvedComparable::Missing,
                [node] => ResolvedComparable::Value((*node).clone()),
                _ => ResolvedComparable::Value(Value::Array(nodes.into_iter().cloned().collect())),
            })
        }
        FilterComparable::Function {
            name,
            args,
            compiled_regex,
        } => match name.as_str() {
            "match" | "search" => {
                let target = resolve_comparable(&args[0], current, root)?;
                let ResolvedComparable::Value(Value::String(target)) = target else {
                    return Ok(ResolvedComparable::Value(Value::Bool(false)));
                };
                let regex = compiled_regex
                    .as_ref()
                    .ok_or_else(|| "Missing compiled regex".to_string())?;
                Ok(ResolvedComparable::Value(Value::Bool(
                    regex.is_match(&target),
                )))
            }
            unsupported => Err(format!("Unsupported function: {unsupported}")),
        },
    }
}
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Array(values) => !values.is_empty(),
        _ => true,
    }
}

fn compare_resolved_values(
    left: &ResolvedComparable,
    op: &FilterOp,
    right: &ResolvedComparable,
) -> bool {
    match (left, right) {
        (ResolvedComparable::Missing, ResolvedComparable::Missing) => matches!(op, FilterOp::Eq),
        (ResolvedComparable::Missing, _) | (_, ResolvedComparable::Missing) => false,
        (ResolvedComparable::Value(left), ResolvedComparable::Value(right)) => {
            compare_dynamic_values(left, op, right)
        }
    }
}

fn compare_dynamic_values(left: &Value, op: &FilterOp, right: &Value) -> bool {
    match (left, right) {
        (Value::String(l), Value::String(r)) => match op {
            FilterOp::Eq => l == r,
            FilterOp::Neq => l != r,
            FilterOp::Lt => l < r,
            FilterOp::Lte => l <= r,
            FilterOp::Gt => l > r,
            FilterOp::Gte => l >= r,
            FilterOp::RegexMatch => Regex::new(r).map(|re| re.is_match(l)).unwrap_or(false),
        },
        (Value::Number(l), Value::Number(r)) => {
            let (Some(lf), Some(rf)) = (l.as_f64(), r.as_f64()) else {
                return false;
            };
            match op {
                FilterOp::Eq => (lf - rf).abs() < f64::EPSILON,
                FilterOp::Neq => (lf - rf).abs() >= f64::EPSILON,
                FilterOp::Lt => lf < rf,
                FilterOp::Lte => lf <= rf,
                FilterOp::Gt => lf > rf,
                FilterOp::Gte => lf >= rf,
                FilterOp::RegexMatch => false,
            }
        }
        (Value::Bool(l), Value::Bool(r)) => match op {
            FilterOp::Eq => l == r,
            FilterOp::Neq => l != r,
            _ => false,
        },
        (Value::Null, Value::Null) => match op {
            FilterOp::Eq => true,
            FilterOp::Neq => false,
            _ => false,
        },
        _ => matches!(op, FilterOp::Neq),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_pest_json_path_extraction() {
        let json = r#"{
            "store": {
                "book": [
                    { "category": "reference", "author": "Nigel Rees", "title": "Sayings of the Century", "price": 8.95 },
                    { "category": "fiction", "author": "Evelyn Waugh", "title": "Sword of Honour", "price": 12.99 }
                ],
                "bicycle": { "color": "red", "price": 19.95 }
            }
        }"#;

        let res = extract_json_path(json, "$.store.book[*].author").unwrap();
        assert_eq!(res, vec![json!("Nigel Rees"), json!("Evelyn Waugh")]);

        let res_single = extract_json_path(json, "$.store.book[0].title").unwrap();
        assert_eq!(res_single, vec![json!("Sayings of the Century")]);

        let res_rec = extract_json_path(json, "$..price").unwrap();
        assert_eq!(res_rec.len(), 3);

        let res_filter = extract_json_path(json, "$.store.book[?(@.price < 10)].title").unwrap();
        assert_eq!(res_filter, vec![json!("Sayings of the Century")]);
    }

    #[test]
    fn test_json_path_rfc9535_slices() {
        let json = r#"[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]"#;

        // Basic slice [1:4]
        let res = extract_json_path(json, "$[1:4]").unwrap();
        assert_eq!(res, vec![json!(1), json!(2), json!(3)]);

        // Slice with step [0:6:2]
        let res_step = extract_json_path(json, "$[0:6:2]").unwrap();
        assert_eq!(res_step, vec![json!(0), json!(2), json!(4)]);

        // Negative slice [:-7]
        let res_neg = extract_json_path(json, "$[:-7]").unwrap();
        assert_eq!(res_neg, vec![json!(0), json!(1), json!(2)]);

        // Reverse slice with negative step [::-1]
        let res_rev = extract_json_path(r#"[1, 2, 3]"#, "$[::-1]").unwrap();
        assert_eq!(res_rev, vec![json!(3), json!(2), json!(1)]);

        // Negative step with bounds [2:0:-1]
        let res_rev_bound = extract_json_path(r#"[1, 2, 3, 4]"#, "$[3:1:-1]").unwrap();
        assert_eq!(res_rev_bound, vec![json!(4), json!(3)]);

        // Step = 0 returns empty per RFC 9535
        let res_zero = extract_json_path(json, "$[0:5:0]").unwrap();
        assert!(res_zero.is_empty());
    }

    #[test]
    fn test_json_path_negative_indices_and_unions() {
        let json = r#"["a", "b", "c", "d", "e"]"#;

        // Negative index [-1]
        let res_last = extract_json_path(json, "$[-1]").unwrap();
        assert_eq!(res_last, vec![json!("e")]);

        // Negative index [-2]
        let res_penult = extract_json_path(json, "$[-2]").unwrap();
        assert_eq!(res_penult, vec![json!("d")]);

        // Union of indices
        let res_union = extract_json_path(json, "$[0, 2, -1]").unwrap();
        assert_eq!(res_union, vec![json!("a"), json!("c"), json!("e")]);
    }

    #[test]
    fn test_json_path_quoted_escaped_member_names() {
        let json = r#"{
            "foo/bar": 1,
            "special \"quote\"": 2,
            "unicode\u0041": 3,
            "emoji\uD83D\uDE00": 4
        }"#;

        let res1 = extract_json_path(json, "$['foo/bar']").unwrap();
        assert_eq!(res1, vec![json!(1)]);

        let res2 = extract_json_path(json, r#"$["special \"quote\""]"#).unwrap();
        assert_eq!(res2, vec![json!(2)]);

        let res3 = extract_json_path(json, r#"$['unicode\u0041']"#).unwrap();
        assert_eq!(res3, vec![json!(3)]);

        // Surrogate pair decoding
        let res4 = extract_json_path(json, r#"$['emoji\uD83D\uDE00']"#).unwrap();
        assert_eq!(res4, vec![json!(4)]);
    }

    #[test]
    fn test_json_path_filter_logical_precedence_and_functions() {
        let json = r#"[
            { "name": "alice", "age": 30, "active": true },
            { "name": "bob", "age": 20, "active": false },
            { "name": "charlie", "age": 40, "active": true }
        ]"#;

        // Existence tests are true for present nodes even when the JSON value is false,
        // so only Charlie satisfies the numeric branch.
        let res = extract_json_path(json, "$[?(!@.active || @.age > 35)].name").unwrap();
        assert_eq!(res, vec![json!("charlie")]);

        // match() function
        let res_match = extract_json_path(json, "$[?(match(@.name, 'a.*'))].name").unwrap();
        assert_eq!(res_match, vec![json!("alice")]);

        // search() function
        let res_search = extract_json_path(json, "$[?(search(@.name, 'li'))].name").unwrap();
        assert_eq!(res_search, vec![json!("alice"), json!("charlie")]);

        // Unsupported function returns error
        let res_unsupported = extract_json_path(json, "$[?(invalid_func(@.name))].name");
        assert!(res_unsupported.is_err());
    }
    #[test]
    fn test_json_path_malformed_syntax_errors() {
        let json = r#"{"a": 1}"#;
        // Unclosed bracket
        assert!(extract_json_path(json, "$[a").is_err());
        // Invalid regex in match()
        assert!(extract_json_path(r#"["test"]"#, "$[?(match(@, '[unclosed'))]").is_err());
        // Empty path or invalid characters
        assert!(extract_json_path(json, "???").is_err());
    }

    #[test]
    fn test_json_path_comparisons_and_null() {
        let json = r#"[
            { "val": null, "flag": false },
            { "val": 42.5, "flag": true },
            { "val": 10.0, "flag": false }
        ]"#;

        let res_null = extract_json_path(json, "$[?(@.val == null)].flag").unwrap();
        assert_eq!(res_null, vec![json!(false)]);

        let res_comp = extract_json_path(json, "$[?(@.val >= 42.5)].flag").unwrap();
        assert_eq!(res_comp, vec![json!(true)]);
    }
    #[test]
    fn missing_property_is_not_literal_null_in_filters() {
        let json = r#"[
            {"value": null},
            {"value": "present"},
            {}
        ]"#;

        let nulls = extract_json_path(json, "$[?(@.value == null)]").unwrap();
        assert_eq!(nulls, vec![json!( {"value": null} )]);

        let missing = extract_json_path(json, "$[?(@.missing)]").unwrap();
        assert!(missing.is_empty());
    }

    #[test]
    fn test_json_path_recursive_descent_with_filter() {
        let json = r#"{
            "level1": {
                "items": [
                    { "id": 1, "target": true },
                    { "id": 2, "target": false }
                ],
                "nested": {
                    "items": [
                        { "id": 3, "target": true }
                    ]
                }
            }
        }"#;

        let res = extract_json_path(json, "$..items[?(@.target == true)].id").unwrap();
        assert_eq!(res, vec![json!(1), json!(3)]);
    }
}
