extern crate core;
extern crate dotenv;

use crate::aws::cloudflare::Cloudflare;
use crate::DynIpError::DomainParse;
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
    let zone_id = std::env::var("CLOUDFLARE_ZONE_ID")?;
    let api_key = std::env::var("CLOUDFLARE_API_KEY")?;
    let email = std::env::var("CLOUDFLARE_EMAIL")?;
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

    let r53 = Cloudflare::new(api_key, zone_id, email, domain_name);
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
