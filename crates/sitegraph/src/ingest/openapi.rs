use crate::model::SitemapObservation;

pub fn observations(
    _document: &[u8],
    _base_url: &str,
    limit: usize,
) -> Result<Vec<SitemapObservation>, String> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    Err("OpenAPI ingestion is not enabled in the sitemap graph MVP".to_owned())
}
