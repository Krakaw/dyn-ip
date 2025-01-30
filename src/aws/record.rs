use aws_sdk_route53::types::{ResourceRecord, ResourceRecordSet, RrType};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Meta {
    pub auto_added: bool,
    pub managed_by_apps: bool,
    pub managed_by_argo_tunnel: bool,
}

#[derive(Deserialize, Debug)]
pub struct CloudflareRecord {
    pub comment: Option<String>,
    pub content: String,
    pub created_on: String,
    pub id: String,
    pub meta: Meta,
    pub modified_on: String,
    pub name: String,
    pub proxiable: bool,
    pub proxied: bool,
    pub settings: serde_json::Value,
    pub tags: Vec<String>,
    pub ttl: u32,
    pub r#type: String,
    pub zone_id: String,
    pub zone_name: String,
}

#[derive(Deserialize, Debug)]
pub struct ResultInfo {
    pub count: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_count: u32,
    pub total_pages: u32,
}

#[derive(Deserialize, Debug)]
pub struct ListRecordsResponse {
    pub errors: Vec<serde_json::Value>,
    pub messages: Vec<serde_json::Value>,
    pub result: Vec<CloudflareRecord>,
    pub result_info: ResultInfo,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub domain: String,
    pub record_type: String,
    pub ip: String,
    pub ttl: i64,
    pub source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayRecord {
    pub domain: String,
    pub record_type: String,
    pub ip: String,
    pub ttl: i64,
    pub id: String,
    pub source_id: String,
}

impl From<CloudflareRecord> for Record {
    fn from(r: CloudflareRecord) -> Self {
        Record {
            domain: r.name,
            record_type: r.r#type,
            ip: r.content,
            ttl: r.ttl as i64,
            source_id: Some(r.id),
        }
    }
}

impl From<&CloudflareRecord> for Record {
    fn from(r: &CloudflareRecord) -> Self {
        Record {
            domain: r.name.clone(),
            record_type: r.r#type.clone(),
            ip: r.content.clone(),
            ttl: r.ttl as i64,
            source_id: Some(r.id.clone()),
        }
    }
}

impl From<&DisplayRecord> for Record {
    fn from(r: &DisplayRecord) -> Self {
        Record {
            domain: r.domain.clone(),
            record_type: r.record_type.clone(),
            ip: r.ip.clone(),
            ttl: r.ttl,
            source_id: Some(r.source_id.clone()),
        }
    }
}
impl Default for Record {
    fn default() -> Self {
        Record {
            domain: "localhost".to_string(),
            record_type: RrType::A.as_str().to_string(),
            ip: "0.0.0.0".to_string(),
            ttl: 60,
            source_id: None,
        }
    }
}
impl From<Record> for ResourceRecordSet {
    fn from(r: Record) -> Self {
        let resource_record_a = ResourceRecord::builder()
            .value(r.ip.to_string())
            .build()
            .unwrap();
        let resource_record_set_a = ResourceRecordSet::builder()
            .name(r.domain)
            .r#type(r.record_type.as_str().into())
            .ttl(r.ttl)
            .resource_records(resource_record_a)
            .build()
            .unwrap();
        resource_record_set_a
    }
}

impl From<&ResourceRecordSet> for Record {
    fn from(r: &ResourceRecordSet) -> Self {
        let r = r.clone();
        Record {
            domain: r.name,
            record_type: r.r#type.as_str().to_string(),
            ip: r
                .resource_records
                .map(|v| v.first().map(|i| i.value.clone()))
                .unwrap_or_default()
                .unwrap_or_default(),
            ttl: r.ttl.unwrap_or(60),
            source_id: None,
        }
    }
}

impl Record {
    pub fn id(&self, salt: &str) -> String {
        let md5 = md5::compute(format!("{}{}", salt, self.domain));
        format!("{:?}", md5)
    }

    pub fn for_display(&self, salt: &str) -> DisplayRecord {
        DisplayRecord {
            domain: self.domain.clone(),
            record_type: self.record_type.clone(),
            ip: self.ip.clone(),
            ttl: self.ttl,
            id: self.id(salt),
            source_id: self.source_id.clone().expect("source_id is required"),
        }
    }
}
