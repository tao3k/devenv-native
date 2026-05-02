use super::DOCUMENT_EXTRACT_ENDPOINTS_ENV;
use crate::gateway::studio::router::handlers::analysis::document_extract::provider::transport::{
    document_extract_endpoint_urls_with_lookup, endpoint_index_for_request,
};

#[test]
fn document_extract_endpoint_urls_default_to_primary_endpoint() {
    let endpoints = document_extract_endpoint_urls_with_lookup("http://127.0.0.1:50051", &|_| None);

    assert_eq!(endpoints, vec!["http://127.0.0.1:50051"]);
}

#[test]
fn document_extract_endpoint_urls_parse_pool_and_deduplicate() {
    let endpoints = document_extract_endpoint_urls_with_lookup("http://default", &|key| {
        (key == DOCUMENT_EXTRACT_ENDPOINTS_ENV)
            .then(|| " http://one:50051/,http://two:50051 ; http://one:50051/ ".to_string())
    });

    assert_eq!(endpoints, vec!["http://one:50051", "http://two:50051"]);
}

#[test]
fn document_extract_endpoint_urls_fall_back_when_pool_is_empty() {
    let endpoints = document_extract_endpoint_urls_with_lookup("http://fallback/", &|key| {
        (key == DOCUMENT_EXTRACT_ENDPOINTS_ENV).then(|| " , ; ".to_string())
    });

    assert_eq!(endpoints, vec!["http://fallback"]);
}

#[test]
fn document_extract_endpoint_index_round_robins_endpoint_pool() -> Result<(), String> {
    assert_eq!(endpoint_index_for_request(0, 3)?, 0);
    assert_eq!(endpoint_index_for_request(1, 3)?, 1);
    assert_eq!(endpoint_index_for_request(2, 3)?, 2);
    assert_eq!(endpoint_index_for_request(3, 3)?, 0);
    assert!(endpoint_index_for_request(0, 0).is_err());
    Ok(())
}
