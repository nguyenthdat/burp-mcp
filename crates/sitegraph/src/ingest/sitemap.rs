use super::html;
use crate::model::SitemapObservation;
use crate::normalize::url::metadata_url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    pub kind: &'static str,
    pub target_url: String,
}

pub fn relationships(observation: &SitemapObservation) -> Vec<Relationship> {
    let mut values = Vec::new();
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
    if observation
        .content_type
        .to_ascii_lowercase()
        .contains("html")
    {
        for reference in html::references(&observation.response_body) {
            push(
                &mut values,
                reference.kind,
                &observation.url,
                &reference.value,
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
