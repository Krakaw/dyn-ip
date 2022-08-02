extern crate dotenv;

use crate::aws::route53::Route53;
use crate::DynIpError::DomainParse;
use aws_sdk_route53::Client;
use dotenv::dotenv;

use crate::error::DynIpError;

mod aws;
mod error;

#[tokio::main]
async fn main() -> Result<(), DynIpError> {
    dotenv().ok();
    let hosted_zone_id = std::env::var("ROUTE_53_ZONE_ID")?;
    let domain_name = addr::parse_domain_name(&std::env::var("DOMAIN_NAME")?)
        .map_err(|e| DomainParse(e.to_string()))?
        .to_string();

    let shared_config = aws_config::from_env().load().await;
    let client = Client::new(&shared_config);
    let r53 = Route53 {
        client: &client,
        hosted_zone_id,
        domain_name,
    };

    Ok(())
}
