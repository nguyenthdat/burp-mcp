use super::{html, javascript};
use crate::model::SitemapObservation;
use crate::normalize::url::metadata_url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub kind: &'static str,
    pub target_url: String,
}

pub fn relationships(observation: &SitemapObservation) -> Vec<Relationship> {
    let mut values = Vec::with_capacity(
        1 + observation.response_links.len()
            + observation.form_actions.len()
            + observation.script_sources.len(),
    );
    push(
        &mut values,
        "redirect",
        &observation.url,
        &observation.redirect_url,
    );
    for value in &observation.response_links {
        push(&mut values, "link", &observation.url, value);
    }
    for value in &observation.form_actions {
        push(&mut values, "form", &observation.url, value);
    }
    for value in &observation.script_sources {
        push(&mut values, "script", &observation.url, value);
    }
    if contains_ascii_case_insensitive(&observation.content_type, "html") {
        for reference in html::references(&observation.response_body) {
            push(
                &mut values,
                reference.kind,
                &observation.url,
                &reference.value,
            );
        }
    }
    if contains_ascii_case_insensitive(&observation.content_type, "javascript") {
        for route in javascript::routes(&observation.response_body) {
            push(
                &mut values,
                "javascript_route",
                &observation.url,
                &route.value,
            );
        }
    }
    values.sort_unstable_by(|left, right| {
        (&left.kind, &left.target_url).cmp(&(&right.kind, &right.target_url))
    });
    values.dedup();
    values.truncate(1_024);
    values
}

fn push(values: &mut Vec<Relationship>, kind: &'static str, base: &str, value: &str) {
    if let Some(target_url) = metadata_url(value, base) {
        values.push(Relationship { kind, target_url });
    }
}

fn contains_ascii_case_insensitive(value: &str, needle: &str) -> bool {
    value
        .as_bytes()
        .windows(needle.len())
        .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
