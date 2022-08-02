use aws_sdk_route53::model::{ResourceRecord, ResourceRecordSet, RrType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub domain: String,
    pub record_type: String,
    pub ip: String,
    pub ttl: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayRecord {
    pub domain: String,
    pub record_type: String,
    pub ip: String,
    pub ttl: i64,
    pub id: String,
}

impl From<&DisplayRecord> for Record {
    fn from(r: &DisplayRecord) -> Self {
        Record {
            domain: r.domain.clone(),
            record_type: r.record_type.clone(),
            ip: r.ip.clone(),
            ttl: r.ttl.clone(),
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
        }
    }
}
impl From<Record> for ResourceRecordSet {
    fn from(r: Record) -> Self {
        let resource_record_a = ResourceRecord::builder().value(r.ip.to_string()).build();
        let resource_record_set_a = ResourceRecordSet::builder()
            .name(r.domain)
            .r#type(r.record_type.as_str().into())
            .ttl(r.ttl)
            .resource_records(resource_record_a)
            .build();
        resource_record_set_a
    }
}

impl From<ResourceRecordSet> for Record {
    fn from(r: ResourceRecordSet) -> Self {
        Record {
            domain: r.name.unwrap_or_default(),
            record_type: r.r#type.unwrap_or_else(|| RrType::A).as_str().to_string(),
            ip: r
                .resource_records
                .map(|v| {
                    v.first()
                        .map(|i| i.value.clone().unwrap_or_else(|| "0.0.0.0".to_string()))
                })
                .unwrap_or_default()
                .unwrap_or_default(),
            ttl: r.ttl.unwrap_or_else(|| 60),
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
            ttl: self.ttl.clone(),
            id: self.id(salt).clone(),
        }
    }
}
