use crate::aws::record::{DisplayRecord, ListRecordsResponse, Record};
use crate::error::DynIpError;
use log::info;
use reqwest::Client;
use serde_json::json;

#[derive(Clone)]
pub struct Cloudflare {
    pub client: Client,
    pub api_key: String,
    pub zone_id: String,
    pub domain_name: String,
}

impl Cloudflare {
    pub fn new(api_key: String, zone_id: String, _email: String, domain_name: String) -> Cloudflare {
        Cloudflare {
            client: Client::new(),
            api_key,
            zone_id,
            domain_name,
        }
    }

    async fn handle_response(&self, response: reqwest::Response) -> Result<reqwest::Response, DynIpError> {
        if response.status().is_success() {
            Ok(response)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            Err(DynIpError::Cloudflare(format!(
                "API request failed with status {}: {}",
                status, error_text
            )))
        }
    }

    pub async fn update_record(&self, record: Record) -> Result<(), DynIpError> {
        let Record {
            record_type,
            domain,
            ip,
            ttl,
            source_id,
        } = record;

        info!("Updating Record: {} {} {} {}", record_type, domain, ip, ttl);

        let source_id = source_id.ok_or(DynIpError::MissingId)?;
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, source_id
        );

        let body = json!({
            "type": record_type,
            "name": domain,
            "content": ip,
            "ttl": ttl,
            "proxied": false
        });

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| DynIpError::Cloudflare(e.to_string()))?;
        
        self.handle_response(response).await?;
        Ok(())
    }

    pub async fn create_record(&self, record: Record) -> Result<(), DynIpError> {
        let Record {
            record_type,
            domain,
            ip,
            ttl,
            source_id: _,
        } = record;

        info!("Creating Record: {} {} {} {}", record_type, domain, ip, ttl);

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );

        let body = json!({
            "type": record_type,
            "name": domain,
            "content": ip,
            "ttl": ttl,
            "proxied": false
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| DynIpError::Cloudflare(e.to_string()))?;
        
        self.handle_response(response).await?;
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
        let mut all_records = Vec::new();
        let mut page = 1;

        loop {
            let mut records = self.fetch_records_page(page).await?;
            let count = records.len();
            all_records.append(&mut records);

            if count == 0 {
                break;
            }
            page += 1;
        }

        Ok(all_records)
    }

    async fn fetch_records_page(&self, page: u32) -> Result<Vec<Record>, DynIpError> {
        info!("Fetching records page {}", page);
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records?page={}",
            self.zone_id, page
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| DynIpError::Cloudflare(e.to_string()))?;
        
        let response = self.handle_response(response).await?;
        
        // Get the response text first for debugging
        let response_text = response.text().await
            .map_err(|e| DynIpError::Cloudflare(format!("Failed to get response text: {}", e)))?;
        
        // Parse the text into JSON
        let list_response = serde_json::from_str::<ListRecordsResponse>(&response_text)
            .map_err(|e| DynIpError::Cloudflare(format!("Failed to decode response: {} - Raw response: {}", e, response_text)))?;
            
        let filtered_records: Vec<Record> = list_response
            .result
            .into_iter()
            .filter(|r| r.r#type == "A" || r.r#type == "CNAME")
            .map(|r| r.into())
            .collect();
            
        info!("Retrieved {} records for page {}", filtered_records.len(), page);
        Ok(filtered_records)
    }

    pub async fn delete_record(&self, salt: &str, id_or_domain: &str) -> Result<(), DynIpError> {
        let records = self.list_display_records(salt).await?;
        if let Some(record) = records
            .iter()
            .find(|r| r.id == id_or_domain || r.domain == id_or_domain)
        {
            info!("Deleting record: {:?}", record);
            let url = format!(
                "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
                self.zone_id, record.source_id
            );

            let response = self
                .client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", &self.api_key))
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(|e| DynIpError::Cloudflare(e.to_string()))?;
            
            self.handle_response(response).await?;
            Ok(())
        } else {
            Err(DynIpError::DomainHashNotFound)
        }
    }
}
