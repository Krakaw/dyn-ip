use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Meta {
    pub auto_added: Option<bool>,
    pub managed_by_apps: Option<bool>,
    pub managed_by_argo_tunnel: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
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
    pub r#type: RecordType,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ResultInfo {
    pub count: u32,
    pub page: u32,
    pub per_page: u32,
    pub total_count: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ListRecordsResponse {
    pub errors: Vec<serde_json::Value>,
    pub messages: Vec<serde_json::Value>,
    pub result: Vec<CloudflareRecord>,
    pub result_info: ResultInfo,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub enum RecordType {
    A,
    Aaaa,
    Cname,
    MX,
    Txt,
}

impl TryFrom<String> for RecordType {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_uppercase().as_str() {
            "A" => Ok(RecordType::A),
            "AAAA" => Ok(RecordType::Aaaa),
            "CNAME" => Ok(RecordType::Cname),
            "MX" => Ok(RecordType::MX),
            "TXT" => Ok(RecordType::Txt),
            _ => Err(format!("Unknown record type: {}", s)),
        }
    }
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordType::A => write!(f, "A"),
            RecordType::Aaaa => write!(f, "AAAA"),
            RecordType::Cname => write!(f, "CNAME"),
            RecordType::MX => write!(f, "MX"),
            RecordType::Txt => write!(f, "TXT"),
        }
    }
}

impl PartialEq<&str> for RecordType {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().eq_ignore_ascii_case(other)
    }
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
            record_type: r.r#type.to_string(),
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
            record_type: r.r#type.to_string(),
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

pub enum RrType {
    A,
    Cname,
}

impl RrType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RrType::A => "A",
            RrType::Cname => "CNAME",
        }
    }

    pub(crate) fn from_str(s: &str) -> Result<RrType, String> {
        match s {
            "A" => Ok(RrType::A),
            "CNAME" => Ok(RrType::Cname),
            _ => Err(format!("Invalid record type: {}", s)),
        }
    }
}
