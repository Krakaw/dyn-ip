use std::net::SocketAddr;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use log::info;

use crate::server::routes;
use crate::{DynIpError, Route53};

#[derive(Clone)]
pub struct ApiConfig {
    pub salt: String,
}

pub async fn start<'a>(
    listen: &SocketAddr,
    route_53: Route53,
    api_config: ApiConfig,
) -> Result<(), DynIpError> {
    info!("Starting server on {:?}", listen);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(api_config.clone()))
            .app_data(web::Data::new(route_53.clone()))
            .service(
                web::scope("/domains")
                    .route("", web::get().to(routes::domains::index))
                    .route(
                        "/{id}",
                        web::patch().to(routes::domains::update_with_peer_address),
                    )
                    .route(
                        "/{id}/{ip}",
                        web::patch().to(routes::domains::update_user_supplied),
                    ),
            )
    })
    .bind(listen)?
    .run()
    .await?;
    Ok(())
}
