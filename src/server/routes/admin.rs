use crate::aws::cloudflare::Cloudflare;
use actix_web::{web, HttpResponse, Responder, Result};

const INDEX_HTML: &str = include_str!("../../../public/index.html");
pub async fn index(route_53: web::Data<Cloudflare>) -> Result<impl Responder> {
    let html = INDEX_HTML.replace("<!--DOMAIN-->", &route_53.domain_name);
    Ok(HttpResponse::Ok().body(html))
}
