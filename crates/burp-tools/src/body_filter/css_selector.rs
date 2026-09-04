use pest::Parser;
use pest_derive::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "grammars/css_selector.pest"]
pub struct CssParser;

use Rule as CssRule;

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

fn unquote_str(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 { &s[1..s.len() - 1] } else { s }
    } else {
        s
    }
}

pub fn parse_css_selector(selector: &str) -> Result<Vec<ParsedCssStep>, String> {
    let pairs = CssParser::parse(CssRule::css_query, selector)
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

#[cfg(test)]
mod tests {
    use super::*;

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
