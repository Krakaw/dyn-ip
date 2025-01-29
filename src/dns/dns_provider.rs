use crate::dns::record::{DisplayRecord, Record};
use crate::error::DynIpError;
use async_trait::async_trait;
use aws_sdk_route53::types::ChangeAction;
use aws_sdk_route53::Client;
#[async_trait]
pub trait DnsProvider: Send + Sync {
    fn domain_name(&self) -> &str;

    async fn update_record(
        &self,
        change_action: ChangeAction,
        record: Record,
    ) -> Result<(), DynIpError>;

    async fn list_display_records(&self, salt: &str) -> Result<Vec<DisplayRecord>, DynIpError>;

    async fn list_records(&self) -> Result<Vec<Record>, DynIpError>;

    async fn delete(&self, salt: &str, id_or_domain: &str) -> Result<(), DynIpError>;
}
