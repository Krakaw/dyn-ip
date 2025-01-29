use std::env::VarError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DynIpError {
    #[error("Env Error: {0}")]
    Env(#[from] VarError),
    #[error("AWS SDK Error: {0}")]
    AwsSdk(String),
    #[error("Domain Parse Error: {0}")]
    DomainParse(String),
    #[error("Socker Error: {0}")]
    SocketAddr(#[from] std::net::AddrParseError),
    #[error("Actix Error: {0}")]
    ActixError(#[from] actix_web::Error),
    #[error("IO Error {0}")]
    FileIO(#[from] std::io::Error),
    #[error("Missing Update IP Address")]
    MissingIp,
    #[error("Missing ID")]
    MissingId,
    #[error("Domain Hash Not Found")]
    DomainHashNotFound,
    #[error("Route53 Error: {0}")]
    Route53(#[from] aws_sdk_route53::Error),
    #[error("Route53 Build Error: {0}")]
    Route53BuildError(String),
    #[error("Cloudflare SDK Error: {0}")]
    CloudflareSdk(String),
    #[error("Unsupported DNS Action")]
    UnsupportedDNSAction,
}

impl actix_web::error::ResponseError for DynIpError {}
