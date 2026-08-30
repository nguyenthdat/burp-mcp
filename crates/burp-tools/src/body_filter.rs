use regex::Regex;
use serde_json::Value;

mod jsonpath_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "jsonpath.pest"]
    pub struct JsonPathParser;
}

mod css_parser {
    use pest_derive::Parser;

    #[derive(Parser)]
    #[grammar = "css.pest"]
    pub struct CssParser;
}

use css_parser::Rule as CssRule;
use jsonpath_parser::Rule as JsonRule;
use pest::Parser;

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
    let pairs = jsonpath_parser::JsonPathParser::parse(JsonRule::json_path, path)
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CssCombinator {
    Descendant,
    Child,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CssAttrOp {
    Exact,
    Contains,
    StartsWith,
    EndsWith,
    WordMatch,
    HyphenPrefix,
    Exists,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CssAttrFilter {
    pub name: String,
    pub op: CssAttrOp,
    pub value: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ParsedCssStep {
    pub combinator: Option<CssCombinator>,
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attrs: Vec<CssAttrFilter>,
    pub pseudo: Option<(String, Option<String>)>,
}

pub fn parse_css_selector(selector: &str) -> Result<Vec<ParsedCssStep>, String> {
    let pairs = css_parser::CssParser::parse(CssRule::css_query, selector)
        .map_err(|e| format!("CSS selector parse error: {e}"))?;

    let mut steps = Vec::new();
    for pair in pairs {
        for step_pair in pair.into_inner() {
            if step_pair.as_rule() == CssRule::selector_step {
                let mut step = ParsedCssStep::default();
                for part in step_pair.into_inner() {
                    match part.as_rule() {
                        CssRule::combinator => match part.as_str().trim() {
                            ">" => step.combinator = Some(CssCombinator::Child),
                            _ => step.combinator = Some(CssCombinator::Descendant),
                        },
                        CssRule::compound_selector => {
                            for comp_part in part.into_inner() {
                                match comp_part.as_rule() {
                                    CssRule::tag_name => {
                                        let t = comp_part.as_str();
                                        if t != "*" {
                                            step.tag = Some(t.to_lowercase());
                                        }
                                    }
                                    CssRule::id_selector => {
                                        let mut id_parts = comp_part.into_inner();
                                        if let Some(id_ident) = id_parts.next() {
                                            step.id = Some(id_ident.as_str().to_string());
                                        }
                                    }
                                    CssRule::class_selector => {
                                        let mut class_parts = comp_part.into_inner();
                                        if let Some(c_ident) = class_parts.next() {
                                            step.classes.push(c_ident.as_str().to_string());
                                        }
                                    }
                                    CssRule::attr_selector => {
                                        let mut attr_parts = comp_part.into_inner();
                                        let attr_name = attr_parts
                                            .next()
                                            .map(|p| p.as_str().to_string())
                                            .unwrap_or_default();
                                        let mut op = CssAttrOp::Exists;
                                        let mut val = None;

                                        if let Some(op_part) = attr_parts.next() {
                                            op = match op_part.as_str() {
                                                "=" => CssAttrOp::Exact,
                                                "*=" => CssAttrOp::Contains,
                                                "^=" => CssAttrOp::StartsWith,
                                                "$=" => CssAttrOp::EndsWith,
                                                "~=" => CssAttrOp::WordMatch,
                                                "|=" => CssAttrOp::HyphenPrefix,
                                                _ => CssAttrOp::Exists,
                                            };
                                            if let Some(val_part) = attr_parts.next() {
                                                val = Some(
                                                    unquote_str(val_part.as_str()).to_string(),
                                                );
                                            }
                                        }

                                        step.attrs.push(CssAttrFilter {
                                            name: attr_name,
                                            op,
                                            value: val,
                                        });
                                    }
                                    CssRule::pseudo_selector => {
                                        let mut pseudo_parts = comp_part.into_inner();
                                        let pseudo_name = pseudo_parts
                                            .next()
                                            .map(|p| p.as_str().to_lowercase())
                                            .unwrap_or_default();
                                        let pseudo_arg = pseudo_parts
                                            .next()
                                            .map(|p| unquote_str(p.as_str()).to_string());
                                        step.pseudo = Some((pseudo_name, pseudo_arg));
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                steps.push(step);
            }
        }
    }
    Ok(steps)
}

pub fn extract_css_selector(html: &str, selector: &str) -> Result<Vec<String>, String> {
    let steps = parse_css_selector(selector)?;
    if steps.is_empty() {
        return Ok(vec![]);
    }

    let mut current_blocks = vec![html.to_string()];
    for step in &steps {
        let mut next_matches = Vec::new();
        for block in &current_blocks {
            let matched = match_css_step(block, step)?;
            next_matches.extend(matched);
        }
        current_blocks = apply_pseudo_filtering(next_matches, &step.pseudo);
    }

    Ok(current_blocks)
}

fn apply_pseudo_filtering(
    elements: Vec<String>,
    pseudo: &Option<(String, Option<String>)>,
) -> Vec<String> {
    let Some((name, arg)) = pseudo else {
        return elements;
    };

    match name.as_str() {
        "first" | "first-child" => {
            if let Some(first) = elements.into_iter().next() {
                vec![first]
            } else {
                vec![]
            }
        }
        "last" | "last-child" => {
            if let Some(last) = elements.into_iter().last() {
                vec![last]
            } else {
                vec![]
            }
        }
        "nth-child" => {
            if let Some(idx) = arg
                .as_deref()
                .and_then(|idx_str| idx_str.trim().parse::<usize>().ok())
                .filter(|&idx| idx >= 1 && idx <= elements.len())
            {
                return vec![elements[idx - 1].clone()];
            }
            vec![]
        }
        "contains" => {
            if let Some(needle) = arg {
                let needle_lower = needle.to_lowercase();
                elements
                    .into_iter()
                    .filter(|elem| elem.to_lowercase().contains(&needle_lower))
                    .collect()
            } else {
                elements
            }
        }
        _ => elements,
    }
}

fn match_css_step(html: &str, step: &ParsedCssStep) -> Result<Vec<String>, String> {
    let tag_pattern = step.tag.as_deref().unwrap_or(r"[a-zA-Z0-9_\-:]+");
    let open_tag_pattern = format!(r"(?is)<({tag_pattern})(\s+[^>]*)?(/)?>");
    let re_open =
        Regex::new(&open_tag_pattern).map_err(|e| format!("Invalid selector regex: {e}"))?;

    let mut results = Vec::new();
    for cap in re_open.captures_iter(html) {
        let whole_open = cap.get(0).map(|m| m.as_str()).unwrap_or_default();
        let matched_tag = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
        let attrs_str = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
        let is_self_closing = cap.get(3).is_some() || whole_open.ends_with("/>");

        if step.id.as_ref().is_some_and(|required_id| {
            !has_attribute_value(attrs_str, "id", &CssAttrOp::Exact, Some(required_id))
        }) {
            continue;
        }

        let mut class_mismatch = false;
        for req_class in &step.classes {
            if !has_class_value(attrs_str, req_class) {
                class_mismatch = true;
                break;
            }
        }
        if class_mismatch {
            continue;
        }

        let mut attr_mismatch = false;
        for attr_filter in &step.attrs {
            if !has_attribute_value(
                attrs_str,
                &attr_filter.name,
                &attr_filter.op,
                attr_filter.value.as_deref(),
            ) {
                attr_mismatch = true;
                break;
            }
        }
        if attr_mismatch {
            continue;
        }

        if is_self_closing || is_void_html_element(matched_tag) {
            results.push(whole_open.trim().to_string());
        } else {
            let start_pos = cap.get(0).map(|m| m.start()).unwrap_or(0);
            let after_open = &html[start_pos..];
            let full_element = extract_balanced_element(after_open, matched_tag);
            results.push(full_element.trim().to_string());
        }
    }

    Ok(results)
}

fn is_void_html_element(tag: &str) -> bool {
    matches!(
        tag.to_lowercase().as_str(),
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn extract_balanced_element<'a>(html_slice: &'a str, tag: &str) -> &'a str {
    let open_pat = format!(r"(?i)<{}\b", regex::escape(tag));
    let close_pat = format!(r"(?i)</{}>", regex::escape(tag));

    let re_open = Regex::new(&open_pat).ok();
    let re_close = Regex::new(&close_pat).ok();

    if let (Some(re_o), Some(re_c)) = (re_open, re_close) {
        let mut depth = 0;
        let mut pos = 0;

        while pos < html_slice.len() {
            let next_open = re_o.find(&html_slice[pos..]);
            let next_close = re_c.find(&html_slice[pos..]);

            match (next_open, next_close) {
                (Some(o), Some(c)) if o.start() < c.start() => {
                    depth += 1;
                    pos += o.end();
                }
                (Some(_), Some(c)) => {
                    depth -= 1;
                    if depth <= 0 {
                        return &html_slice[..pos + c.end()];
                    }
                    pos += c.end();
                }
                (None, Some(c)) => {
                    depth -= 1;
                    if depth <= 0 {
                        return &html_slice[..pos + c.end()];
                    }
                    pos += c.end();
                }
                _ => break,
            }
        }
    }

    let close_tag = format!("</{tag}>");
    if let Some(end_offset) = html_slice.to_lowercase().find(&close_tag.to_lowercase()) {
        &html_slice[..end_offset + close_tag.len()]
    } else {
        html_slice
    }
}

fn has_attribute_value(
    attrs: &str,
    name: &str,
    op: &CssAttrOp,
    expected_val: Option<&str>,
) -> bool {
    let pattern = format!(
        r#"(?i)\b{}\s*(?:=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?"#,
        regex::escape(name)
    );

    let Ok(re) = Regex::new(&pattern) else {
        return false;
    };

    let Some(caps) = re.captures(attrs) else {
        return false;
    };

    if *op == CssAttrOp::Exists {
        return true;
    }

    let actual_val = caps
        .get(1)
        .or_else(|| caps.get(2))
        .or_else(|| caps.get(3))
        .map(|m| m.as_str())
        .unwrap_or_default();

    let expected = expected_val.unwrap_or_default();

    match op {
        CssAttrOp::Exact => actual_val.eq_ignore_ascii_case(expected),
        CssAttrOp::Contains => actual_val.to_lowercase().contains(&expected.to_lowercase()),
        CssAttrOp::StartsWith => actual_val
            .to_lowercase()
            .starts_with(&expected.to_lowercase()),
        CssAttrOp::EndsWith => actual_val
            .to_lowercase()
            .ends_with(&expected.to_lowercase()),
        CssAttrOp::WordMatch => actual_val
            .split_whitespace()
            .any(|w| w.eq_ignore_ascii_case(expected)),
        CssAttrOp::HyphenPrefix => {
            actual_val.eq_ignore_ascii_case(expected)
                || actual_val
                    .to_lowercase()
                    .starts_with(&format!("{}-", expected.to_lowercase()))
        }
        CssAttrOp::Exists => true,
    }
}

fn has_class_value(attrs: &str, expected_class: &str) -> bool {
    let pattern = r#"(?i)\bclass\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#;
    Regex::new(pattern)
        .ok()
        .and_then(|re| re.captures(attrs))
        .and_then(|caps| caps.get(1).or_else(|| caps.get(2)).or_else(|| caps.get(3)))
        .is_some_and(|val| {
            val.as_str()
                .split_whitespace()
                .any(|c| c.eq_ignore_ascii_case(expected_class))
        })
}

pub fn extract_headers_only(http_text: &str) -> String {
    let newline = if http_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let delimiter = format!("{newline}{newline}");
    if let Some((head, _)) = http_text.split_once(&delimiter) {
        head.to_string()
    } else {
        http_text.to_string()
    }
}

pub fn is_binary_mime_type(mime: &str) -> bool {
    let mime = mime.to_lowercase();
    mime.starts_with("image/")
        || mime.starts_with("video/")
        || mime.starts_with("audio/")
        || mime.starts_with("font/")
        || mime.contains("octet-stream")
        || mime.contains("pdf")
        || mime.contains("zip")
        || mime.contains("gzip")
        || mime.contains("tar")
        || mime.contains("protobuf")
        || mime.contains("wasm")
}

pub fn is_binary_payload(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }
    let sample = &bytes[..bytes.len().min(1024)];
    let null_count = sample.iter().filter(|&&b| b == 0).count();
    null_count > 0
        || sample
            .iter()
            .filter(|&&b| b < 9 || (b > 13 && b < 32))
            .count()
            > sample.len() / 10
}

pub fn filter_and_truncate_payload(
    bytes: &[u8],
    content_type: Option<&str>,
    headers_only: bool,
    extract_css: Option<&str>,
    extract_json: Option<&str>,
    max_length: Option<usize>,
) -> (String, bool) {
    if bytes.is_empty() {
        return (String::new(), false);
    }

    let raw_text = String::from_utf8_lossy(bytes).into_owned();

    if headers_only {
        return (extract_headers_only(&raw_text), false);
    }

    if let Some(mime) = content_type {
        if is_binary_mime_type(mime) {
            let len = bytes.len();
            return (format!("<binary data: {len} bytes [{mime}]>"), false);
        }
    } else if is_binary_payload(bytes) {
        let len = bytes.len();
        return (format!("<binary data: {len} bytes>"), false);
    }

    let newline = if raw_text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let (_head, body) = raw_text
        .split_once(&format!("{newline}{newline}"))
        .unwrap_or(("", &raw_text));

    if let Some((css_sel, matches)) =
        extract_css.and_then(|sel| extract_css_selector(body, sel).ok().map(|m| (sel, m)))
    {
        let formatted = format!(
            "--- CSS Matches for `{css_sel}` ({}) ---\n{}",
            matches.len(),
            matches.join("\n")
        );
        return (formatted, false);
    }

    if let Some((json_path, matches)) =
        extract_json.and_then(|path| extract_json_path(body, path).ok().map(|m| (path, m)))
    {
        let json_out = serde_json::to_string_pretty(&matches).unwrap_or_else(|_| "[]".to_string());
        let formatted = format!(
            "--- JSONPath Matches for `{json_path}` ({}) ---\n{}",
            matches.len(),
            json_out
        );
        return (formatted, false);
    }

    if let Some(max_len) = max_length.filter(|&len| raw_text.len() > len) {
        let truncated = format!(
            "{}\n\n... [truncated {} bytes]",
            &raw_text[..max_len.min(raw_text.len())],
            raw_text.len().saturating_sub(max_len)
        );
        return (truncated, true);
    }

    (raw_text, false)
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

    #[test]
    fn test_pest_css_selector() {
        let html = r#"
        <html>
            <body>
                <div class="container">
                    <form id="login" action="/login" method="POST">
                        <input type="hidden" name="csrf" value="secret123" />
                        <input type="text" name="username" class="form-control" />
                        <button type="submit" class="btn btn-primary">Log In</button>
                    </form>
                    <div class="alert alert-danger">Invalid credentials</div>
                    <a href="https://example.com/forgot">Forgot Password</a>
                </div>
            </body>
        </html>
        "#;

        let res = extract_css_selector(html, "form#login").unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].contains("csrf"));

        let res_input = extract_css_selector(html, "input[name=csrf]").unwrap();
        assert_eq!(res_input.len(), 1);
        assert!(res_input[0].contains("secret123"));

        let res_class = extract_css_selector(html, ".alert").unwrap();
        assert_eq!(res_class.len(), 1);
        assert!(res_class[0].contains("Invalid credentials"));

        // Multi-class
        let res_btn = extract_css_selector(html, ".btn.btn-primary").unwrap();
        assert_eq!(res_btn.len(), 1);
        assert!(res_btn[0].contains("Log In"));

        // Attribute starts-with
        let res_href = extract_css_selector(html, "a[href^='https://']").unwrap();
        assert_eq!(res_href.len(), 1);
        assert!(res_href[0].contains("Forgot Password"));

        // Pseudo :contains
        let res_contains = extract_css_selector(html, "button:contains('Log In')").unwrap();
        assert_eq!(res_contains.len(), 1);
    }
}
