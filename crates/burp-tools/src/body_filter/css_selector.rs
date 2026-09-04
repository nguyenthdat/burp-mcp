use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammars/css_selector.pest"]
pub struct CssParser;

use Rule as CssRule;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CssCombinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
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
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CssPseudo {
    FirstChild,
    LastChild,
    OnlyChild,
    NthChild { a: isize, b: isize },
    First,
    Last,
    Contains(String),
    Not(Vec<ComplexSelector>),
    Is(Vec<ComplexSelector>),
    Where(Vec<ComplexSelector>),
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ParsedCssStep {
    pub combinator: Option<CssCombinator>,
    pub tag: Option<String>,
    pub id: Option<String>,
    pub classes: Vec<String>,
    pub attrs: Vec<CssAttrFilter>,
    pub pseudos: Vec<CssPseudo>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ComplexSelector {
    pub steps: Vec<ParsedCssStep>,
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SelectorList {
    pub selectors: Vec<ComplexSelector>,
}

pub fn unescape_css(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next_c) = chars.peek() {
                if next_c.is_ascii_hexdigit() {
                    let mut hex = String::new();
                    while let Some(&hc) = chars.peek() {
                        if hc.is_ascii_hexdigit() && hex.len() < 6 {
                            hex.push(hc);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if let Some(&sc) = chars.peek()
                        && (sc == ' ' || sc == '\t')
                    {
                        chars.next();
                    }
                    if let Ok(cp) = u32::from_str_radix(&hex, 16)
                        && let Some(unicode_char) = char::from_u32(cp)
                    {
                        out.push(unicode_char);
                        continue;
                    }
                    out.push('\u{FFFD}');
                } else if next_c != '\r' && next_c != '\n' && next_c != '\x0c' {
                    out.push(next_c);
                    chars.next();
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn unquote_and_unescape(s: &str) -> String {
    let trimmed = s.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            unescape_css(&trimmed[1..trimmed.len() - 1])
        } else {
            unescape_css(trimmed)
        }
    } else {
        unescape_css(trimmed)
    }
}

pub fn parse_nth_expr(s: &str) -> Result<(isize, isize), String> {
    let s = s.trim().to_lowercase();
    if s == "odd" {
        return Ok((2, 1));
    }
    if s == "even" {
        return Ok((2, 0));
    }
    if !s.contains('n') {
        let b = s
            .parse::<isize>()
            .map_err(|e| format!("Invalid nth integer: {e}"))?;
        return Ok((0, b));
    }

    let parts: Vec<&str> = s.split('n').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid nth expression: {s}"));
    }

    let a_str = parts[0].trim();
    let a = if a_str.is_empty() || a_str == "+" {
        1
    } else if a_str == "-" {
        -1
    } else {
        a_str
            .parse::<isize>()
            .map_err(|e| format!("Invalid nth multiplier: {e}"))?
    };

    let b_part = parts[1].replace([' ', '\t'], "");
    let b = if b_part.is_empty() {
        0
    } else {
        b_part
            .parse::<isize>()
            .map_err(|e| format!("Invalid nth offset: {e}"))?
    };

    Ok((a, b))
}

pub fn parse_css_selector_list(selector: &str) -> Result<SelectorList, String> {
    let pairs = CssParser::parse(CssRule::css_query, selector)
        .map_err(|e| format!("CSS selector parse error: {e}"))?;

    let mut list = SelectorList::default();
    for pair in pairs {
        if pair.as_rule() == CssRule::css_query {
            for inner in pair.into_inner() {
                if inner.as_rule() == CssRule::selector_list {
                    for complex_pair in inner.into_inner() {
                        if complex_pair.as_rule() == CssRule::complex_selector {
                            let mut complex = ComplexSelector::default();
                            let mut next_combinator: Option<CssCombinator> = None;

                            for item in complex_pair.into_inner() {
                                match item.as_rule() {
                                    CssRule::compound_selector => {
                                        let mut step = parse_compound_selector(item)?;
                                        step.combinator = next_combinator.take();
                                        complex.steps.push(step);
                                    }
                                    CssRule::combinator => {
                                        let c_str = item.as_str().trim();
                                        let comb = match c_str {
                                            ">" => CssCombinator::Child,
                                            "+" => CssCombinator::NextSibling,
                                            "~" => CssCombinator::SubsequentSibling,
                                            _ => CssCombinator::Descendant,
                                        };
                                        next_combinator = Some(comb);
                                    }
                                    _ => {}
                                }
                            }
                            list.selectors.push(complex);
                        }
                    }
                }
            }
        }
    }

    if list.selectors.is_empty() {
        return Err("Empty selector list".to_string());
    }

    Ok(list)
}

fn parse_compound_selector(pair: pest::iterators::Pair<CssRule>) -> Result<ParsedCssStep, String> {
    let mut step = ParsedCssStep::default();

    for part in pair.into_inner() {
        match part.as_rule() {
            CssRule::type_or_universal => {
                for inner in part.into_inner() {
                    match inner.as_rule() {
                        CssRule::tag_name => {
                            let raw = inner.as_str();
                            let unesc = unescape_css(raw);
                            step.tag = Some(unesc.to_lowercase());
                        }
                        CssRule::universal => {
                            // '*' matches any tag; None in step.tag
                        }
                        _ => {}
                    }
                }
            }
            CssRule::subclass_selector => {
                for sub in part.into_inner() {
                    match sub.as_rule() {
                        CssRule::id_selector => {
                            for id_ident in sub.into_inner() {
                                if id_ident.as_rule() == CssRule::ident {
                                    step.id = Some(unescape_css(id_ident.as_str()));
                                }
                            }
                        }
                        CssRule::class_selector => {
                            for class_ident in sub.into_inner() {
                                if class_ident.as_rule() == CssRule::ident {
                                    step.classes.push(unescape_css(class_ident.as_str()));
                                }
                            }
                        }
                        CssRule::attr_selector => {
                            let mut attr_name = String::new();
                            let mut op = CssAttrOp::Exists;
                            let mut val = None;
                            let mut case_sensitive = None;

                            for attr_part in sub.into_inner() {
                                match attr_part.as_rule() {
                                    CssRule::attr_name => {
                                        attr_name = unescape_css(attr_part.as_str());
                                    }
                                    CssRule::attr_matcher => {
                                        op = match attr_part.as_str() {
                                            "=" => CssAttrOp::Exact,
                                            "*=" => CssAttrOp::Contains,
                                            "^=" => CssAttrOp::StartsWith,
                                            "$=" => CssAttrOp::EndsWith,
                                            "~=" => CssAttrOp::WordMatch,
                                            "|=" => CssAttrOp::HyphenPrefix,
                                            _ => CssAttrOp::Exists,
                                        };
                                    }
                                    CssRule::attr_val => {
                                        val = Some(unquote_and_unescape(attr_part.as_str()));
                                    }
                                    CssRule::attr_flag => {
                                        let flag = attr_part.as_str().to_lowercase();
                                        if flag == "i" {
                                            case_sensitive = Some(false);
                                        } else if flag == "s" {
                                            case_sensitive = Some(true);
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            step.attrs.push(CssAttrFilter {
                                name: attr_name,
                                op,
                                value: val,
                                case_sensitive,
                            });
                        }
                        CssRule::pseudo_selector => {
                            let Some(pseudo_part) = sub.into_inner().next() else {
                                continue;
                            };
                            match pseudo_part.as_rule() {
                                CssRule::pseudo_ident => {
                                    let name = pseudo_part.as_str().to_lowercase();
                                    let pseudo = match name.as_str() {
                                        "first-child" => CssPseudo::FirstChild,
                                        "last-child" => CssPseudo::LastChild,
                                        "only-child" => CssPseudo::OnlyChild,
                                        "first" => CssPseudo::First,
                                        "last" => CssPseudo::Last,
                                        _ => {
                                            return Err(format!(
                                                "unsupported pseudo-class: {name}"
                                            ));
                                        }
                                    };
                                    step.pseudos.push(pseudo);
                                }
                                CssRule::nth_pseudo => {
                                    let expression = pseudo_part
                                        .into_inner()
                                        .find(|part| part.as_rule() == CssRule::nth_expr)
                                        .ok_or("nth-child requires an An+B expression")?;
                                    let (a, b) = parse_nth_expr(expression.as_str())?;
                                    step.pseudos.push(CssPseudo::NthChild { a, b });
                                }
                                CssRule::selector_pseudo => {
                                    let mut parts = pseudo_part.into_inner();
                                    let name = parts
                                        .next()
                                        .ok_or("selector pseudo requires a name")?
                                        .as_str()
                                        .to_lowercase();
                                    let selector = parts
                                        .next()
                                        .ok_or("selector pseudo requires a selector list")?
                                        .as_str();
                                    let selectors = parse_css_selector_list(selector)?.selectors;
                                    let pseudo = match name.as_str() {
                                        "not" => CssPseudo::Not(selectors),
                                        "is" => CssPseudo::Is(selectors),
                                        "where" => CssPseudo::Where(selectors),
                                        _ => {
                                            return Err(format!(
                                                "unsupported selector pseudo: {name}"
                                            ));
                                        }
                                    };
                                    step.pseudos.push(pseudo);
                                }
                                CssRule::string_pseudo => {
                                    let argument = pseudo_part
                                        .into_inner()
                                        .find(|part| part.as_rule() == CssRule::string_lit)
                                        .ok_or("contains requires a quoted string")?;
                                    step.pseudos.push(CssPseudo::Contains(unquote_and_unescape(
                                        argument.as_str(),
                                    )));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(step)
}

pub fn parse_css_selector(selector: &str) -> Result<Vec<ParsedCssStep>, String> {
    let list = parse_css_selector_list(selector)?;
    if let Some(first) = list.selectors.into_iter().next() {
        Ok(first.steps)
    } else {
        Ok(vec![])
    }
}

// ---------------- HTML Node Tree & Tokenizer ----------------

#[derive(Debug, Clone)]
pub struct HtmlElement {
    pub id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub tag: String,
    pub attributes: Vec<(String, String)>,
    pub text: String,
    pub outer_html: String,
}

impl HtmlElement {
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn has_class(&self, class_name: &str) -> bool {
        if let Some(cls) = self.get_attr("class") {
            cls.split_whitespace().any(|c| c == class_name)
        } else {
            false
        }
    }
}

#[derive(Debug, Default)]
pub struct HtmlDocument {
    pub nodes: Vec<HtmlElement>,
    pub root_children: Vec<usize>,
}

fn is_void_element(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
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

fn is_raw_text_element(tag: &str) -> bool {
    matches!(tag.to_ascii_lowercase().as_str(), "script" | "style")
}

pub fn parse_html_document(html: &str) -> HtmlDocument {
    const MAX_ELEMENTS: usize = 50_000;
    const MAX_DEPTH: usize = 512;

    let mut doc = HtmlDocument::default();
    let chars: Vec<char> = html.chars().collect();
    let len = chars.len();
    let mut pos = 0;
    let mut stack: Vec<(usize, usize)> = Vec::new(); // (node_id, start_char_pos)

    while pos < len && doc.nodes.len() < MAX_ELEMENTS {
        if chars[pos] == '<' {
            let start = pos;
            pos += 1;

            // Comment <!-- ... -->
            if pos + 2 < len && chars[pos] == '!' && chars[pos + 1] == '-' && chars[pos + 2] == '-'
            {
                pos += 3;
                while pos + 2 < len {
                    if chars[pos] == '-' && chars[pos + 1] == '-' && chars[pos + 2] == '>' {
                        pos += 3;
                        break;
                    }
                    pos += 1;
                }
                continue;
            }

            // DOCTYPE or CDATA
            if pos < len && chars[pos] == '!' {
                while pos < len && chars[pos] != '>' {
                    pos += 1;
                }
                if pos < len {
                    pos += 1;
                }
                continue;
            }

            // Close tag </tag>
            if pos < len && chars[pos] == '/' {
                pos += 1;
                let tag_start = pos;
                while pos < len
                    && (chars[pos].is_alphanumeric()
                        || chars[pos] == '-'
                        || chars[pos] == '_'
                        || chars[pos] == ':')
                {
                    pos += 1;
                }
                let close_tag: String = chars[tag_start..pos].iter().collect();
                let close_tag = close_tag.to_ascii_lowercase();

                while pos < len && chars[pos] != '>' {
                    pos += 1;
                }
                if pos < len {
                    pos += 1;
                }

                if let Some(idx) = stack.iter().rposition(|&(node_id, _)| {
                    doc.nodes[node_id].tag.eq_ignore_ascii_case(&close_tag)
                }) {
                    for i in (idx..stack.len()).rev() {
                        let (closed_id, start_char) = stack[i];
                        let outer: String = chars[start_char..pos].iter().collect();
                        doc.nodes[closed_id].outer_html = outer;
                    }
                    stack.truncate(idx);
                }
                continue;
            }

            // Open tag <tag attrs...>
            let tag_start = pos;
            while pos < len
                && (chars[pos].is_alphanumeric()
                    || chars[pos] == '-'
                    || chars[pos] == '_'
                    || chars[pos] == ':')
            {
                pos += 1;
            }
            if tag_start == pos {
                continue;
            }
            let tag_name: String = chars[tag_start..pos].iter().collect();
            let tag_name_lower = tag_name.to_ascii_lowercase();

            // Parse attributes
            let mut attributes = Vec::new();
            let mut is_self_closing = false;

            while pos < len && chars[pos] != '>' {
                if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '>' {
                    is_self_closing = true;
                    pos += 2;
                    break;
                }
                if chars[pos].is_whitespace() {
                    pos += 1;
                    continue;
                }

                // Attribute name
                let attr_name_start = pos;
                while pos < len
                    && !chars[pos].is_whitespace()
                    && chars[pos] != '='
                    && chars[pos] != '>'
                    && chars[pos] != '/'
                {
                    pos += 1;
                }
                if attr_name_start == pos {
                    pos += 1;
                    continue;
                }
                let attr_name: String = chars[attr_name_start..pos].iter().collect();

                while pos < len && chars[pos].is_whitespace() {
                    pos += 1;
                }

                let mut attr_val = String::new();
                if pos < len && chars[pos] == '=' {
                    pos += 1;
                    while pos < len && chars[pos].is_whitespace() {
                        pos += 1;
                    }
                    if pos < len && (chars[pos] == '"' || chars[pos] == '\'') {
                        let quote = chars[pos];
                        pos += 1;
                        let val_start = pos;
                        while pos < len && chars[pos] != quote {
                            pos += 1;
                        }
                        attr_val = chars[val_start..pos].iter().collect();
                        if pos < len {
                            pos += 1; // skip quote
                        }
                    } else {
                        let val_start = pos;
                        while pos < len && !chars[pos].is_whitespace() && chars[pos] != '>' {
                            pos += 1;
                        }
                        attr_val = chars[val_start..pos].iter().collect();
                    }
                }

                attributes.push((attr_name, attr_val));
            }

            if pos < len && chars[pos] == '>' {
                pos += 1;
            }

            let node_id = doc.nodes.len();
            let parent_id = stack.last().map(|&(pid, _)| pid);

            let is_void = is_void_element(&tag_name_lower) || is_self_closing;
            let outer_html: String = chars[start..pos].iter().collect();

            doc.nodes.push(HtmlElement {
                id: node_id,
                parent: parent_id,
                children: Vec::new(),
                tag: tag_name_lower.clone(),
                attributes,
                text: String::new(),
                outer_html,
            });

            if let Some(pid) = parent_id {
                doc.nodes[pid].children.push(node_id);
            } else {
                doc.root_children.push(node_id);
            }

            if is_void {
                continue;
            }

            // Raw text elements: script, style
            if is_raw_text_element(&tag_name_lower) {
                let close_str = format!("</{}>", tag_name_lower);
                let close_chars: Vec<char> = close_str.chars().collect();
                let text_start = pos;

                while pos + close_chars.len() <= len {
                    let mut matches = true;
                    for (k, &cc) in close_chars.iter().enumerate() {
                        if !chars[pos + k].eq_ignore_ascii_case(&cc) {
                            matches = false;
                            break;
                        }
                    }
                    if matches {
                        let script_text: String = chars[text_start..pos].iter().collect();
                        doc.nodes[node_id].text = script_text;
                        pos += close_chars.len();
                        let outer: String = chars[start..pos].iter().collect();
                        doc.nodes[node_id].outer_html = outer;
                        break;
                    }
                    pos += 1;
                }
                continue;
            }

            if stack.len() < MAX_DEPTH {
                stack.push((node_id, start));
            }
        } else {
            // Text node
            let text_start = pos;
            while pos < len && chars[pos] != '<' {
                pos += 1;
            }
            if let Some(&(parent_id, _)) = stack.last() {
                let txt: String = chars[text_start..pos].iter().collect();
                doc.nodes[parent_id].text.push_str(&txt);
            }
        }
    }

    // Unclosed tags on stack
    for &(unclosed_id, start_char) in &stack {
        let outer: String = chars[start_char..len].iter().collect();
        doc.nodes[unclosed_id].outer_html = outer;
    }

    doc
}

// ---------------- CSS Selector Evaluator ----------------

pub fn extract_css_selector(html: &str, selector: &str) -> Result<Vec<String>, String> {
    let selector_list = parse_css_selector_list(selector)?;
    let doc = parse_html_document(html);

    let mut matched_node_ids = Vec::new();

    for complex in &selector_list.selectors {
        let matches = match_complex_selector(&doc, complex);
        for id in matches {
            if !matched_node_ids.contains(&id) {
                matched_node_ids.push(id);
            }
        }
    }

    // Preserve document order
    matched_node_ids.sort_unstable();

    let results = matched_node_ids
        .into_iter()
        .map(|id| doc.nodes[id].outer_html.trim().to_string())
        .collect();

    Ok(results)
}

fn match_complex_selector(doc: &HtmlDocument, complex: &ComplexSelector) -> Vec<usize> {
    if complex.steps.is_empty() {
        return vec![];
    }

    let mut current_candidates: Vec<usize> = (0..doc.nodes.len()).collect();

    let first_step = &complex.steps[0];
    current_candidates.retain(|&id| matches_compound_selector(doc, id, first_step));

    for step in &complex.steps[1..] {
        let mut next_candidates = Vec::new();
        let comb = step
            .combinator
            .as_ref()
            .unwrap_or(&CssCombinator::Descendant);

        for &curr_id in &current_candidates {
            match comb {
                CssCombinator::Child => {
                    for &child_id in &doc.nodes[curr_id].children {
                        if matches_compound_selector(doc, child_id, step)
                            && !next_candidates.contains(&child_id)
                        {
                            next_candidates.push(child_id);
                        }
                    }
                }
                CssCombinator::Descendant => {
                    let mut descendants = Vec::new();
                    collect_all_descendants(doc, curr_id, &mut descendants);
                    for desc_id in descendants {
                        if matches_compound_selector(doc, desc_id, step)
                            && !next_candidates.contains(&desc_id)
                        {
                            next_candidates.push(desc_id);
                        }
                    }
                }
                CssCombinator::NextSibling => {
                    let siblings = doc.nodes[curr_id]
                        .parent
                        .map_or(doc.root_children.as_slice(), |parent_id| {
                            doc.nodes[parent_id].children.as_slice()
                        });
                    if let Some(pos) = siblings.iter().position(|&sid| sid == curr_id)
                        && let Some(&next_sibling) = siblings.get(pos + 1)
                        && matches_compound_selector(doc, next_sibling, step)
                        && !next_candidates.contains(&next_sibling)
                    {
                        next_candidates.push(next_sibling);
                    }
                }
                CssCombinator::SubsequentSibling => {
                    let siblings = doc.nodes[curr_id]
                        .parent
                        .map_or(doc.root_children.as_slice(), |parent_id| {
                            doc.nodes[parent_id].children.as_slice()
                        });
                    if let Some(pos) = siblings.iter().position(|&sid| sid == curr_id) {
                        for &subsequent_sibling in &siblings[pos + 1..] {
                            if matches_compound_selector(doc, subsequent_sibling, step)
                                && !next_candidates.contains(&subsequent_sibling)
                            {
                                next_candidates.push(subsequent_sibling);
                            }
                        }
                    }
                }
            }
        }
        current_candidates = next_candidates;
    }

    current_candidates
}

fn collect_all_descendants(doc: &HtmlDocument, root_id: usize, out: &mut Vec<usize>) {
    for &child_id in &doc.nodes[root_id].children {
        out.push(child_id);
        collect_all_descendants(doc, child_id, out);
    }
}

fn matches_compound_selector(doc: &HtmlDocument, node_id: usize, step: &ParsedCssStep) -> bool {
    let node = &doc.nodes[node_id];

    // Tag check
    if let Some(required_tag) = &step.tag
        && !node.tag.eq_ignore_ascii_case(required_tag)
    {
        return false;
    }

    // ID check
    if let Some(required_id) = &step.id
        && node.get_attr("id") != Some(required_id.as_str())
    {
        return false;
    }

    // Classes check
    for req_class in &step.classes {
        if !node.has_class(req_class) {
            return false;
        }
    }

    // Attributes check
    for attr in &step.attrs {
        let Some(actual_val) = node.get_attr(&attr.name) else {
            return false;
        };

        if attr.op == CssAttrOp::Exists {
            continue;
        }

        let expected = attr.value.as_deref().unwrap_or_default();
        let is_match = match attr.case_sensitive {
            Some(true) => match attr.op {
                CssAttrOp::Exact => actual_val == expected,
                CssAttrOp::Contains => actual_val.contains(expected),
                CssAttrOp::StartsWith => actual_val.starts_with(expected),
                CssAttrOp::EndsWith => actual_val.ends_with(expected),
                CssAttrOp::WordMatch => actual_val.split_whitespace().any(|w| w == expected),
                CssAttrOp::HyphenPrefix => {
                    actual_val == expected || actual_val.starts_with(&format!("{expected}-"))
                }
                CssAttrOp::Exists => true,
            },
            _ => {
                let actual_lower = actual_val.to_ascii_lowercase();
                let exp_lower = expected.to_ascii_lowercase();
                match attr.op {
                    CssAttrOp::Exact => actual_lower == exp_lower,
                    CssAttrOp::Contains => actual_lower.contains(&exp_lower),
                    CssAttrOp::StartsWith => actual_lower.starts_with(&exp_lower),
                    CssAttrOp::EndsWith => actual_lower.ends_with(&exp_lower),
                    CssAttrOp::WordMatch => actual_val
                        .split_whitespace()
                        .any(|w| w.eq_ignore_ascii_case(expected)),
                    CssAttrOp::HyphenPrefix => {
                        actual_lower == exp_lower
                            || actual_lower.starts_with(&format!("{exp_lower}-"))
                    }
                    CssAttrOp::Exists => true,
                }
            }
        };

        if !is_match {
            return false;
        }
    }

    // Pseudos check
    for pseudo in &step.pseudos {
        if !matches_pseudo(doc, node_id, pseudo) {
            return false;
        }
    }

    true
}

fn matches_pseudo(doc: &HtmlDocument, node_id: usize, pseudo: &CssPseudo) -> bool {
    let node = &doc.nodes[node_id];

    match pseudo {
        CssPseudo::FirstChild => {
            if let Some(parent_id) = node.parent {
                doc.nodes[parent_id].children.first() == Some(&node_id)
            } else {
                doc.root_children.first() == Some(&node_id)
            }
        }
        CssPseudo::LastChild => {
            if let Some(parent_id) = node.parent {
                doc.nodes[parent_id].children.last() == Some(&node_id)
            } else {
                doc.root_children.last() == Some(&node_id)
            }
        }
        CssPseudo::OnlyChild => {
            if let Some(parent_id) = node.parent {
                doc.nodes[parent_id].children.len() == 1
            } else {
                doc.root_children.len() == 1
            }
        }
        CssPseudo::NthChild { a, b } => {
            let siblings = if let Some(parent_id) = node.parent {
                &doc.nodes[parent_id].children
            } else {
                &doc.root_children
            };

            if let Some(pos) = siblings.iter().position(|&sid| sid == node_id) {
                let idx = (pos + 1) as isize;
                if *a == 0 {
                    idx == *b
                } else {
                    let diff = idx - *b;
                    (diff % *a == 0) && (diff / *a >= 0)
                }
            } else {
                false
            }
        }
        CssPseudo::First => node_id == 0,
        CssPseudo::Last => node_id + 1 == doc.nodes.len(),
        CssPseudo::Contains(needle) => {
            let needle_lower = needle.to_ascii_lowercase();
            node.outer_html.to_ascii_lowercase().contains(&needle_lower)
        }
        CssPseudo::Not(selectors) => !selectors.iter().any(|complex| {
            let matches = match_complex_selector(doc, complex);
            matches.contains(&node_id)
        }),
        CssPseudo::Is(selectors) | CssPseudo::Where(selectors) => selectors.iter().any(|complex| {
            let matches = match_complex_selector(doc, complex);
            matches.contains(&node_id)
        }),
    }
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

    #[test]
    fn test_css_combinators_and_lists() {
        let html = r#"
        <div id="main">
            <h1>Title</h1>
            <p class="intro">Paragraph 1</p>
            <p class="body">Paragraph 2</p>
            <span class="note">Note</span>
        </div>
        "#;

        // Child combinator >
        let res = extract_css_selector(html, "div#main > p").unwrap();
        assert_eq!(res.len(), 2);

        // Next sibling +
        let res_next = extract_css_selector(html, "h1 + p").unwrap();
        assert_eq!(res_next.len(), 1);
        assert!(res_next[0].contains("Paragraph 1"));

        // Subsequent sibling ~
        let res_sub = extract_css_selector(html, "h1 ~ p").unwrap();
        assert_eq!(res_sub.len(), 2);

        // Selector list ,
        let res_list = extract_css_selector(html, "h1, span.note").unwrap();
        assert_eq!(res_list.len(), 2);
    }

    #[test]
    fn test_css_escapes_and_unicode() {
        let html = r#"
        <div id="item:1">First</div>
        <div class="foo.bar">Second</div>
        <div data-emoji="🦀">Crab</div>
        "#;

        let res1 = extract_css_selector(html, r#"#item\:1"#).unwrap();
        assert_eq!(res1.len(), 1);

        let res2 = extract_css_selector(html, r#".foo\.bar"#).unwrap();
        assert_eq!(res2.len(), 1);

        let res3 = extract_css_selector(html, r#"[data-emoji="🦀"]"#).unwrap();
        assert_eq!(res3.len(), 1);
    }

    #[test]
    fn test_css_attributes_case_flags() {
        let html = r#"
        <input name="testCASE" value="foo" />
        <input name="other" value="FOO" />
        "#;

        // Case-sensitive 's' flag
        let res_s = extract_css_selector(html, r#"input[value="foo" s]"#).unwrap();
        assert_eq!(res_s.len(), 1);

        // Case-insensitive 'i' flag
        let res_i = extract_css_selector(html, r#"input[value="foo" i]"#).unwrap();
        assert_eq!(res_i.len(), 2);
    }

    #[test]
    fn test_css_nth_child_and_logical_pseudos() {
        let html = r#"
        <ul>
            <li class="item">One</li>
            <li class="item">Two</li>
            <li class="item special">Three</li>
            <li class="item">Four</li>
        </ul>
        "#;

        let res_first = extract_css_selector(html, "li:first-child").unwrap();
        assert_eq!(res_first.len(), 1);
        assert!(res_first[0].contains("One"));

        let res_last = extract_css_selector(html, "li:last-child").unwrap();
        assert_eq!(res_last.len(), 1);
        assert!(res_last[0].contains("Four"));

        let res_odd = extract_css_selector(html, "li:nth-child(odd)").unwrap();
        assert_eq!(res_odd.len(), 2);

        let res_not = extract_css_selector(html, "li:not(.special)").unwrap();
        assert_eq!(res_not.len(), 3);

        let res_is = extract_css_selector(html, "li:is(.special)").unwrap();
        assert_eq!(res_is.len(), 1);
    }
    #[test]
    fn logical_pseudo_accepts_type_selectors_starting_with_n() {
        let html = "<main><nav>menu</nav><p>copy</p></main>";

        let not_nav = extract_css_selector(html, "main > :not(nav)").unwrap();
        assert_eq!(not_nav, vec!["<p>copy</p>"]);

        let is_nav = extract_css_selector(html, "main > :is(nav)").unwrap();
        assert_eq!(is_nav, vec!["<nav>menu</nav>"]);
    }

    #[test]
    fn nth_child_supports_n_offsets_and_root_siblings() {
        let nested = "<ul><li>1</li><li>2</li><li>3</li><li>4</li></ul>";
        let nth = extract_css_selector(nested, "li:nth-child(n+2)").unwrap();
        assert_eq!(nth, vec!["<li>2</li>", "<li>3</li>", "<li>4</li>"]);

        let root = "<h1>title</h1><p>body</p><p>tail</p>";
        let adjacent = extract_css_selector(root, "h1 + p").unwrap();
        assert_eq!(adjacent, vec!["<p>body</p>"]);
        let following = extract_css_selector(root, "h1 ~ p").unwrap();
        assert_eq!(following, vec!["<p>body</p>", "<p>tail</p>"]);
    }

    #[test]
    fn test_css_attributes_with_brackets_and_quotes() {
        let html = r#"
        <div data-formula="a > b && c < d">Formula</div>
        <script>var x = "<div id='fake'></div>";</script>
        "#;

        let res = extract_css_selector(html, "div[data-formula]").unwrap();
        assert_eq!(res.len(), 1);

        // Ensure script contents are not matched as DOM elements
        let res_fake = extract_css_selector(html, "#fake").unwrap();
        assert_eq!(res_fake.len(), 0);
    }
    #[test]
    fn test_css_whitespace_descendant_vs_compound() {
        let html = r#"
        <div class="target">
            <span class="target">Inner</span>
        </div>
        <div class="other">
            <span class="target">Outside</span>
        </div>
        "#;

        // Compound: div.target (matches outer div only)
        let res_compound = extract_css_selector(html, "div.target").unwrap();
        assert_eq!(res_compound.len(), 1);
        assert!(res_compound[0].starts_with("<div"));

        // Descendant with whitespace: div .target (matches span inside div)
        let res_descendant = extract_css_selector(html, "div .target").unwrap();
        assert_eq!(res_descendant.len(), 2);
    }

    #[test]
    fn test_css_empty_attributes_and_void_elements() {
        let html = r#"
        <form>
            <input type="text" disabled />
            <input type="hidden" value="" />
            <img src="pic.jpg" alt="photo" />
            <p>Text after img</p>
        </form>
        "#;

        // Empty / boolean attribute
        let res_disabled = extract_css_selector(html, "input[disabled]").unwrap();
        assert_eq!(res_disabled.len(), 1);

        let res_empty_val = extract_css_selector(html, "input[value='']").unwrap();
        assert_eq!(res_empty_val.len(), 1);

        // Void element: img is closed immediately, so p is its sibling, not child
        let res_sibling = extract_css_selector(html, "img + p").unwrap();
        assert_eq!(res_sibling.len(), 1);
        assert!(res_sibling[0].contains("Text after img"));
    }

    #[test]
    fn test_css_comments_and_malformed_html() {
        let html = r#"
        <div>
            <!-- <div class="hidden-ghost">Fake</div> -->
            <p>Real Content</p>
        </div>
        "#;

        // Comment should not be parsed as element
        let res_ghost = extract_css_selector(html, ".hidden-ghost").unwrap();
        assert_eq!(res_ghost.len(), 0);

        let res_p = extract_css_selector(html, "div > p").unwrap();
        assert_eq!(res_p.len(), 1);
    }

    #[test]
    fn test_css_invalid_syntax_error() {
        let res = extract_css_selector("<div></div>", "div[unclosed");
        assert!(res.is_err());

        let res2 = extract_css_selector("<div></div>", "div > > p");
        assert!(res2.is_err());
    }

    #[test]
    fn test_css_structural_and_logical_where() {
        let html = r#"
        <div id="parent">
            <span class="only">Only Span</span>
        </div>
        <div id="parent2">
            <span class="first">First</span>
            <span class="second">Second</span>
        </div>
        "#;

        let res_only = extract_css_selector(html, "span:only-child").unwrap();
        assert_eq!(res_only.len(), 1);
        assert!(res_only[0].contains("Only Span"));

        let res_where = extract_css_selector(html, "span:where(.first, .only)").unwrap();
        assert_eq!(res_where.len(), 2);
    }
}
