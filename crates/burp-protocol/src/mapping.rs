use super::{PageRequest, ProxyHistoryQuery};
use super::protocol as proto;

pub(crate) fn page_request(page: PageRequest) -> proto::PageRequest {
    proto::PageRequest {
        limit: page.limit,
        cursor: page.cursor,
    }
}

pub(crate) fn proxy_history_request(query: ProxyHistoryQuery) -> proto::ProxyHistoryRequest {
    proto::ProxyHistoryRequest {
        page: Some(page_request(query.page)),
        url_filter: query.url_filter,
        method_filter: query.method_filter,
        status_filter: query.status_filter,
        has_notes: query.has_notes,
        color: query.color,
    }
}
