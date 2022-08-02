use crate::aws::record::{DisplayRecord, Record};
use crate::DynIpError;
use aws_sdk_route53::model::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use aws_sdk_route53::Client;

#[derive(Clone)]
pub struct Route53 {
    pub client: Client,
    pub hosted_zone_id: String,
    pub domain_name: String,
}

impl Route53 {
    pub async fn update_record(
        &self,
        change_action: ChangeAction,
        record: Record,
    ) -> Result<(), DynIpError> {
        let change = Change::builder()
            .action(change_action)
            .resource_record_set(record.into())
            .build();
        let change_batch = ChangeBatch::builder().changes(change).build();
        self.client
            .change_resource_record_sets()
            .hosted_zone_id(self.hosted_zone_id.clone())
            .change_batch(change_batch)
            .send()
            .await
            .map_err(|e| DynIpError::AwsSdk(e.to_string()))?;

        Ok(())
    }

    pub async fn list_display_records(&self, salt: &str) -> Result<Vec<DisplayRecord>, DynIpError> {
        Ok(self
            .list_records()
            .await?
            .iter()
            .map(|r| r.for_display(salt))
            .collect::<Vec<DisplayRecord>>())
    }

    pub async fn list_records(&self) -> Result<Vec<Record>, DynIpError> {
        let mut result = vec![];
        let mut next_page = None;
        loop {
            let output = self
                .client
                .list_resource_record_sets()
                .hosted_zone_id(self.hosted_zone_id.clone())
                .set_start_record_identifier(next_page.clone())
                .send()
                .await
                .map_err(|e| DynIpError::AwsSdk(e.to_string()))?;
            for record in output.resource_record_sets.unwrap_or_default() {
                result.push(record.into())
            }
            next_page = output.next_record_identifier;
            if next_page.is_none() {
                break;
            }
        }
        Ok(result)
    }
}
