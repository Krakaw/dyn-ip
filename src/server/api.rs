use std::net::SocketAddr;

use actix_web::dev::ServiceRequest;
use actix_web::middleware::{Condition, Logger};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::{basic, AuthenticationError};
use actix_web_httpauth::middleware::HttpAuthentication;
use log::info;

use crate::server::auth::Auth;
use crate::server::routes;
use crate::{DynIpError, Route53};

#[derive(Clone)]
pub struct ApiConfig {
    pub salt: String,
    pub auth: Auth,
}

async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if let Some(api_config) = req.app_data::<web::Data<ApiConfig>>() {
        if api_config.auth.check_credentials(credentials) {
            Ok(req)
        } else {
            let config = req
                .app_data::<basic::Config>()
                .cloned()
                .unwrap_or_default()
                .realm("dyn-ip requires auth");
            Err((AuthenticationError::from(config).into(), req))
        }
    } else {
        panic!("ApiConfig data not found.")
    }
}

pub async fn start<'a>(
    listen: &SocketAddr,
    route_53: Route53,
    api_config: ApiConfig,
) -> Result<(), DynIpError> {
    info!("Starting server on {:?}", listen);
    HttpServer::new(move || {
        let auth = HttpAuthentication::basic(validator);

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(api_config.clone()))
            .app_data(web::Data::new(route_53.clone()))
            .service(
                web::scope("/api")
                    .wrap(Condition::new(api_config.auth.has_credentials(), auth))
                    .route(
                        "/admin",
                        web::get().to(|| async {
                            HttpResponse::Ok().body(include_str!("../../public/index.html"))
                        }),
                    )
                    .service(
                        web::scope("/domains")
                            .route("", web::get().to(routes::domains::index))
                            .route("", web::post().to(routes::domains::add))
                            .route(
                                "/{id}",
                                web::patch().to(routes::domains::update_with_peer_address),
                            )
                            .route(
                                "/{id}/{ip}",
                                web::patch().to(routes::domains::update_user_supplied),
                            ),
                    ),
            )
            .route(
                "/",
                web::get().to(|req: HttpRequest| async move {
                    let ip = req
                        .peer_addr()
                        .map(|p| p.ip().to_string())
                        .unwrap_or("".to_string());
                    HttpResponse::Ok().body(ip)
                }),
            )
    })
    .bind(listen)?
    .run()
    .await?;
    Ok(())
}
