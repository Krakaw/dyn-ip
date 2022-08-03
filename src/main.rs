extern crate dotenv;

use crate::aws::route53::Route53;
use crate::DynIpError::DomainParse;
use aws_sdk_route53::Client;
use dotenv::dotenv;
use env_logger::Env;
use std::net::SocketAddr;

use crate::error::DynIpError;
use crate::server::api::ApiConfig;
use crate::server::auth::Auth;

mod aws;
mod error;
mod server;

#[tokio::main]
async fn main() -> Result<(), DynIpError> {
    dotenv().ok();
    env_logger::Builder::from_env(Env::default().default_filter_or("dyn_ip=info")).init();
    let hosted_zone_id = std::env::var("ROUTE_53_ZONE_ID")?;
    let domain_name = addr::parse_domain_name(&std::env::var("DOMAIN_NAME")?)
        .map_err(|e| DomainParse(e.to_string()))?
        .to_string();
    let listen: SocketAddr = std::env::var("LISTEN")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()?;
    let salt = std::env::var("SALT").unwrap_or_default();
    let username = std::env::var("BASIC_AUTH_USERNAME")
        .ok()
        .filter(|u| !u.is_empty());
    let password = std::env::var("BASIC_AUTH_PASSWORD")
        .ok()
        .filter(|p| !p.is_empty());

    let shared_config = aws_config::from_env().load().await;
    let client = Client::new(&shared_config);
    let r53 = Route53 {
        client,
        hosted_zone_id,
        domain_name,
    };
    server::api::start(
        &listen,
        r53,
        ApiConfig {
            salt,
            auth: Auth { username, password },
        },
    )
    .await?;

    Ok(())
}
