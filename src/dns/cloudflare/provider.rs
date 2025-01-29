use crate::dns::dns_provider::DnsProvider;
use crate::dns::record::{DisplayRecord, Record};
use crate::{DomainParse, DynIpError};
use addr::parse_domain_name;
use async_trait::async_trait;
use aws_sdk_route53::types::ChangeAction;
use cloudflare::endpoints::dns::ListDnsRecords;
use cloudflare::endpoints::dns::{CreateDnsRecord, DeleteDnsRecord, DnsContent};
use cloudflare::framework::auth::Credentials;
use cloudflare::framework::response::ApiFailure;
use cloudflare::framework::{HttpApiClient, HttpApiClientConfig};
use log::info;

pub struct CloudflareDNS {
    pub client: HttpApiClient,
    pub zone_id: String,
    pub domain_name: String,
}

impl CloudflareDNS {
    pub fn new(
        email: String,
        api_key: String,
        zone_id: String,
        domain_name: String,
    ) -> Result<CloudflareDNS, DynIpError> {
        let credentials = Credentials::UserAuthKey {
            email,
            key: api_key,
        };
        let client = HttpApiClient::new(credentials, HttpApiClientConfig::default())
            .map_err(|e| DynIpError::CloudflareSdk(e.to_string()))?;

        let domain_name = if !domain_name.ends_with('.') {
            format!("{}.", domain_name)
        } else {
            domain_name
        };
        parse_domain_name(&domain_name).map_err(|e| DomainParse(e.to_string()))?;
        Ok(CloudflareDNS {
            client,
            zone_id,
            domain_name,
        })
    }
}

#[async_trait]
impl DnsProvider for CloudflareDNS {
    fn domain_name(&self) -> &str {
        &self.domain_name
    }

    async fn update_record(
        &self,
        change_action: ChangeAction,
        record: Record,
    ) -> Result<(), DynIpError> {
        info!("Updating Record: {:?} {:?}", change_action, record);
        match change_action {
            ChangeAction::Upsert => {
                let content = DnsContent::A {
                    content: record.value,
                };
                self.client
                    .request(&CreateDnsRecord {
                        zone_identifier: &self.zone_id,
                        params: cloudflare::endpoints::dns::DnsRecordParams {
                            name: &record.domain,
                            content,
                            ttl: Some(record.ttl as u32),
                            proxied: Some(false),
                        },
                    })
                    .await
                    .map_err(|e| DynIpError::CloudflareSdk(format!("{:?}", e)))?;
            }
            ChangeAction::Delete => {
                // Assume `record.id` holds the Cloudflare DNS record ID
                self.client
                    .request(&DeleteDnsRecord {
                        zone_identifier: &self.zone_id,
                        identifier: &record.id,
                    })
                    .await
                    .map_err(|e| DynIpError::CloudflareSdk(format!("{:?}", e)))?;
            }
            _ => return Err(DynIpError::UnsupportedDNSAction),
        }
        Ok(())
    }

    async fn list_display_records(&self, salt: &str) -> Result<Vec<DisplayRecord>, DynIpError> {
        let records = self.list_records().await?;
        Ok(records
            .iter()
            .map(|r| r.for_display(salt))
            .collect::<Vec<DisplayRecord>>())
    }

    async fn list_records(&self) -> Result<Vec<Record>, DynIpError> {
        let response = self
            .client
            .request(&ListDnsRecords {
                zone_identifier: &self.zone_id,
                params: None,
            })
            .await;

        match response {
            Ok(res) => Ok(res.result.iter().map(Record::from).collect()),
            Err(ApiFailure::Error(_, api_errors)) => {
                Err(DynIpError::CloudflareSdk(api_errors.to_string()))
            }
            Err(e) => Err(DynIpError::CloudflareSdk(format!("{:?}", e))),
        }
    }

    async fn delete(&self, salt: &str, id_or_domain: &str) -> Result<(), DynIpError> {
        let records = self.list_display_records(salt).await?;
        if let Some(record) = records
            .iter()
            .find(|r| r.id == id_or_domain || r.domain == id_or_domain)
        {
            info!("Deleting record: {:?}", record);
            self.update_record(ChangeAction::Delete, record.into())
                .await?;
            Ok(())
        } else {
            Err(DynIpError::DomainHashNotFound)
        }
    }
}
