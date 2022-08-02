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
}
