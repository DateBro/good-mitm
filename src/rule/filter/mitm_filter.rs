use http_mitm::{
    async_trait::async_trait,
    hyper::{Body, Request},
    HttpContext,
};

#[derive(Clone, Default)]
pub struct MitmFilter {}

#[async_trait]
impl http_mitm::MitmFilter for MitmFilter {
    async fn filter(&mut self, _ctx: &HttpContext, _req: &Request<Body>) -> bool {
        true
    }
}