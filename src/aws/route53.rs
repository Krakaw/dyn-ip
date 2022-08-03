use crate::aws::record::{DisplayRecord, Record};
use crate::{DomainParse, DynIpError};
use addr::parse_domain_name;
use aws_sdk_route53::model::{Change, ChangeAction, ChangeBatch};
use aws_sdk_route53::Client;

#[derive(Clone)]
pub struct Route53 {
    pub client: Client,
    pub hosted_zone_id: String,
    pub domain_name: String,
}

impl Route53 {
    pub fn new(
        client: Client,
        hosted_zone_id: String,
        domain_name: String,
    ) -> Result<Route53, DynIpError> {
        let domain_name = if !domain_name.ends_with('.') {
            format!("{}.", domain_name)
        } else {
            domain_name
        };
        parse_domain_name(&domain_name).map_err(|e| DomainParse(e.to_string()))?;
        Ok(Route53 {
            client,
            hosted_zone_id,
            domain_name,
        })
    }
    pub async fn update_record(
        &self,
        change_action: ChangeAction,
        record: Record,
    ) -> Result<(), DynIpError> {
        let domain_name = self.domain_name.clone();
        let record = if !record.domain.ends_with(&domain_name) {
            Record {
                domain: format!("{}.{}", record.domain, domain_name),
                ..record
            }
        } else {
            record
        };
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
        let domain_name = self.domain_name.clone();
        loop {
            let output = self
                .client
                .list_resource_record_sets()
                .hosted_zone_id(self.hosted_zone_id.clone())
                .set_start_record_identifier(next_page.clone())
                .send()
                .await
                .map_err(|e| DynIpError::AwsSdk(e.to_string()))?;
            for record in output
                .resource_record_sets
                .unwrap_or_default()
                .iter()
                .filter(|r| r.name != Some(domain_name.clone()))
            {
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
