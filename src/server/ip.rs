use actix_web::HttpRequest;

pub fn get_ip_from_request(req: &HttpRequest) -> Option<String> {
    let headers = req.headers();
    let ip = headers
        .get("X-Real-IP")
        .or_else(|| headers.get("X-Forwarded-For"))
        .and_then(|h| h.to_str().ok().map(|s| s.to_string()));
    ip
}
