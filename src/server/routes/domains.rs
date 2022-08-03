use crate::aws::record::Record;
use crate::DynIpError::{DomainHashNotFound, MissingIp};
use crate::{ApiConfig, DomainParse, DynIpError, Route53};
use actix_web::{web, HttpRequest, Responder, Result};
use addr::parse_domain_name;
use aws_sdk_route53::model::ChangeAction;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::IpAddr;

pub async fn index(
    route_53: web::Data<Route53>,
    config: web::Data<ApiConfig>,
) -> Result<impl Responder> {
    let records = route_53.list_display_records(&config.salt).await?;
    Ok(web::Json(records))
}

pub async fn update_with_peer_address(
    route_53: web::Data<Route53>,
    config: web::Data<ApiConfig>,
    id: web::Path<String>,
    req: HttpRequest,
) -> Result<impl Responder> {
    let ip = req
        .peer_addr()
        .map(|p| p.ip().to_string())
        .ok_or(MissingIp)?;
    _update_inner(route_53, config, id.into_inner(), ip).await
}

pub async fn update_user_supplied(
    route_53: web::Data<Route53>,
    config: web::Data<ApiConfig>,
    id_ip: web::Path<(String, IpAddr)>,
) -> Result<impl Responder> {
    let (id, ip) = id_ip.into_inner();
    _update_inner(route_53, config, id, ip.to_string()).await
}

async fn _update_inner(
    route_53: web::Data<Route53>,
    config: web::Data<ApiConfig>,
    id: String,
    ip: String,
) -> Result<impl Responder> {
    let records = route_53.list_display_records(&config.salt).await?;

    if let Some(record) = records.iter().find(|r| r.id == id) {
        let mut record: Record = record.into();
        record.ip = ip;
        route_53
            .update_record(ChangeAction::Upsert, record.clone())
            .await?;
        let display_record = record.for_display(&config.salt);
        return Ok(web::Json(display_record));
    }
    Err(DomainHashNotFound.into())
}

#[derive(Deserialize)]
pub struct AddQuery {
    pub domain: String,
    pub ip: Option<IpAddr>,
}

pub async fn add(
    req: HttpRequest,
    route_53: web::Data<Route53>,
    config: web::Data<ApiConfig>,
    domain_ip: web::Query<AddQuery>,
) -> Result<impl Responder> {
    let domain_ip = domain_ip.into_inner();
    let ip = domain_ip
        .ip
        .ok_or_else(|| req.peer_addr().map(|p| p.ip()))
        .ok()
        .map(|ip| ip.to_string())
        .ok_or(DynIpError::MissingIp)?;
    let domain = parse_domain_name(&domain_ip.domain)
        .map_err(|e| DomainParse(e.to_string()))?
        .to_string();
    let record = Record {
        domain,
        ip,
        ..Record::default()
    };
    route_53
        .update_record(ChangeAction::Upsert, record.clone())
        .await?;

    Ok(web::Json(record.for_display(&config.salt)))
}
