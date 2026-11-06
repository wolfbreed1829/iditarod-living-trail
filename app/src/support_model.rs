use serde::{Deserialize, Serialize};

pub const KINDS: &[(&str, &str, &str)] = &[
    ("giving", "Giving enquiry", "donation_url"),
    ("membership", "Membership enquiry", "membership_url"),
    ("volunteer", "Volunteer offer", "volunteer_url"),
    ("newsletter", "Newsletter interest", "newsletter_url"),
];
pub const AREAS: &[(&str, &str)] = &[("where-needed", "Where help is needed"), ("trail-care", "Trail care & access"), ("history", "History & oral accounts"), ("education", "Education & outreach")];
pub const CONSENT_VERSION: &str = "support-2026-11-06";
pub fn label(kind: &str) -> &'static str { KINDS.iter().find(|(k,_,_)|*k==kind).map(|(_,label,_)|*label).unwrap_or("Support request") }
pub fn area_label(area: &str) -> &'static str { AREAS.iter().find(|(k,_)|*k==area).map(|(_,label)|*label).unwrap_or("Where help is needed") }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Submission {
    pub key: String, pub kind: String, pub name: String, pub email: String,
    pub area: String, pub amount: String, pub availability: String, pub message: String,
    pub consent: bool, #[serde(default)] pub website: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SupportRecord {
    pub id: u64, pub reference: String, pub kind: String, pub name: String, pub email: String,
    pub area: String, pub amount: String, pub availability: String, pub message: String,
    pub consent_version: String, pub created: String, pub updated: String,
    pub status: String, pub note: String, pub version: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Receipt {
    pub reference: String, pub kind: String, pub area: String, pub created: String, pub withdrawn: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SupportChange { pub version: u64, pub status: String, pub note: String, #[serde(default)] pub delete: bool }
