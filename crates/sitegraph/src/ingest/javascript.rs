use regex::Regex;
use std::sync::LazyLock;

const MAX_INPUT_BYTES: usize = 512 * 1_024;
const MAX_ROUTES: usize = 512;
const MAX_ROUTE_BYTES: usize = 8 * 1_024;

static ROUTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?x)
        (?:fetch\s*\(|axios(?:\.[A-Za-z]+)?\s*\(|\.open\s*\(\s*["'][A-Z]+["']\s*,\s*)?
        ["'`](?P<route>/(?:api|rest|graphql|socket\.io|ws|v[0-9]+)(?:[^"'`\\]|\\.)*)["'`]"#,
    )
    .expect("static JavaScript route regex must compile")
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub value: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

pub fn routes(input: &[u8]) -> Vec<Route> {
    let bounded = &input[..input.len().min(MAX_INPUT_BYTES)];
    let text = String::from_utf8_lossy(bounded);
    let mut routes = ROUTE
        .captures_iter(&text)
        .filter_map(|captures| {
            let found = captures.name("route")?;
            if found.is_empty() || found.len() > MAX_ROUTE_BYTES {
                return None;
            }
            Some(Route {
                value: found.as_str().replace("\\/", "/"),
                byte_start: found.start(),
                byte_end: found.end(),
            })
        })
        .take(MAX_ROUTES)
        .collect::<Vec<_>>();
    routes.sort_unstable_by(|left, right| left.value.cmp(&right.value));
    routes.dedup_by(|left, right| left.value == right.value);
    routes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_routes_without_executing_javascript() {
        let input =
            br#"fetch('/api/users'); axios.post(`/v1/login`); const ignored = '/images/a.png';"#;
        let routes = routes(input);
        assert_eq!(
            routes
                .iter()
                .map(|route| route.value.as_str())
                .collect::<Vec<_>>(),
            vec!["/api/users", "/v1/login"]
        );
        for route in routes {
            assert_eq!(
                &input[route.byte_start..route.byte_end],
                route.value.as_bytes()
            );
        }
    }

    #[test]
    fn input_and_result_count_are_bounded() {
        let input = (0..600)
            .map(|index| format!("fetch('/api/{index}');"))
            .collect::<String>();
        assert_eq!(routes(input.as_bytes()).len(), MAX_ROUTES);
        assert!(routes(&vec![b'a'; MAX_INPUT_BYTES + 1]).is_empty());
    }
}
