use crate::rule::{self, Rule};
use http_mitm::{
    async_trait::async_trait,
    hyper::{header, Body, Request, Response, Uri},
    HttpContext, HttpHandler, RequestOrResponse,
};
use log::info;

#[derive(Clone, Default)]
pub struct MitmHandler {
    should_modify_response: bool,
    rule: Vec<Rule>,
    uri: Option<Uri>,
}

#[async_trait]
impl HttpHandler for MitmHandler {
    async fn handle_request(
        &mut self,
        _ctx: &HttpContext,
        req: Request<Body>,
    ) -> RequestOrResponse {
        self.uri = Some(req.uri().clone());

        // remove accept-encoding to avoid encoded body
        let mut req = req;
        req.headers_mut().remove(header::ACCEPT_ENCODING);

        let rules = rule::match_rules(&req);
        if !rules.is_empty() {
            self.should_modify_response = true;
        }

        for mut rule in rules {
            self.rule.push(rule.clone());
            let rt = rule.do_req(req).await;
            if let RequestOrResponse::Request(r) = rt {
                req = r;
            } else {
                return rt;
            }
        }

        RequestOrResponse::Request(req)
    }

    async fn handle_response(&mut self, _ctx: &HttpContext, res: Response<Body>) -> Response<Body> {
        if !self.should_modify_response || self.rule.is_empty() {
            return res;
        }
        let uri = self.uri.as_ref().unwrap();
        let content_type = match res.headers().get(header::CONTENT_TYPE) {
            Some(content_type) => content_type.to_str().unwrap_or_default(),
            None => "unknown",
        };
        info!(
            "[Response] {} {} {}",
            res.status(),
            uri.host().unwrap_or_default(),
            content_type
        );

        let mut res = res;
        let rules = self.rule.clone();
        for rule in rules {
            res = rule.do_res(res).await;
        }
        res
    }
}
