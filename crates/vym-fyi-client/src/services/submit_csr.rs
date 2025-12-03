use vym_fyi_model::models::errors::AppResult;
use vym_fyi_model::models::sign_csr_request::SignCsrRequest;
use vym_fyi_model::models::signed_cert_response::SignedCertResponse;
use vym_fyi_model::services::http_client::HttpClient;

/// Submit a CSR to the server for signing
pub async fn submit_csr(
    http: &HttpClient,
    endpoint: &str,
    token: &str,
    csr_pem: &str,
) -> AppResult<SignedCertResponse> {
    let dto = SignCsrRequest {
        csr_pem: csr_pem.to_string(),
    };
    http.post_json_auth(endpoint, token, &dto).await
}
