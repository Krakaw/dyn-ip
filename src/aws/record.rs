use aws_sdk_route53::model::{ResourceRecord, ResourceRecordSet, RrType};
use std::net::IpAddr;
use std::str::FromStr;

use uuid::Uuid;

#[derive(Debug)]
pub struct Record {
    pub domain: String,
    pub record_type: RrType,
    pub ip: String,
    pub ttl: i64,
    pub id: Option<Uuid>,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            domain: "localhost".to_string(),
            record_type: RrType::A,
            ip: "0.0.0.0".to_string(),
            ttl: 60,
            id: Some(Uuid::new_v4()),
        }
    }
}
impl From<Record> for ResourceRecordSet {
    fn from(r: Record) -> Self {
        let resource_record_a = ResourceRecord::builder().value(r.ip.to_string()).build();
        let resource_record_set_a = ResourceRecordSet::builder()
            .name(r.domain)
            .r#type(r.record_type)
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
            record_type: r.r#type.unwrap_or_else(|| RrType::A),
            ip: r
                .resource_records
                .map(|v| {
                    v.first()
                        .map(|i| i.value.clone().unwrap_or_else(|| "0.0.0.0".to_string()))
                })
                .unwrap_or_default()
                .unwrap_or_default(),
            ttl: r.ttl.unwrap_or_else(|| 60),
            id: None,
        }
    }
}
