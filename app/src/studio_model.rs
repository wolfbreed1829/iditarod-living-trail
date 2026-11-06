use crate::cms::Content;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Draft {
    pub title: String,
    pub summary: String,
    pub body: String,
    pub slug: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
}
impl Draft {
    pub fn field(&self, key: &str) -> String {
        self.fields.get(key).cloned().unwrap_or_default()
    }
    pub fn from_content(c: &Content) -> Self {
        Self {
            title: c.title.clone(),
            summary: c.summary.clone(),
            body: c.body.clone(),
            slug: c.slug.clone(),
            fields: c.fields.clone(),
        }
    }
    pub fn apply(&self, c: &mut Content) {
        c.title = self.title.clone();
        c.summary = self.summary.clone();
        c.body = self.body.clone();
        c.slug = self.slug.clone();
        c.fields = self.fields.clone();
        c.path = content_path(&c.kind, &c.slug);
    }
}
pub fn content_path(kind: &str, slug: &str) -> String {
    if kind == "page" {
        if slug == "home" {
            "/".into()
        } else {
            format!("/{slug}")
        }
    } else {
        format!(
            "/{}/{slug}",
            match kind {
                "story" => "stories",
                "place" => "places",
                "resource" => "resources",
                "event" => "events",
                _ => "campaigns",
            }
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Revision {
    pub id: String,
    pub at: String,
    pub by: String,
    pub label: String,
    pub content: Draft,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub at: String,
    pub content: Draft,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(flatten)]
    pub content: Content,
    pub status: String,
    pub modified: String,
    pub draft: Option<Draft>,
    pub schedule: Option<Schedule>,
    pub version: u64,
    pub author: String,
    #[serde(default)]
    pub owner: u64,
    #[serde(default)]
    pub ever_published: bool,
    #[serde(default)]
    pub history: Vec<Revision>,
}
impl Item {
    pub fn editable(&self) -> Draft {
        self.draft
            .clone()
            .unwrap_or_else(|| Draft::from_content(&self.content))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub login: String,
    pub name: String,
    pub role: String,
}
impl User {
    pub fn publisher(&self) -> bool {
        self.role == "admin" || self.role == "editor"
    }
    pub fn can_edit(&self, i: &Item) -> bool {
        self.publisher() || i.owner == self.id
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Session {
    pub user: User,
    pub csrf: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Workspace {
    pub items: Vec<Item>,
    pub media: Vec<Media>,
    pub connections: HashMap<String, String>,
    pub users: Vec<User>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Media {
    pub id: String,
    pub name: String,
    pub url: String,
    pub mime: String,
    pub size: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change {
    pub action: String,
    pub version: u64,
    #[serde(default)]
    pub content: Option<Draft>,
    #[serde(default)]
    pub at: Option<String>,
    #[serde(default)]
    pub revision: Option<String>,
}
pub const KINDS: &[(&str, &str)] = &[
    ("page", "Pages"),
    ("story", "Stories"),
    ("place", "Places"),
    ("resource", "Documents"),
    ("event", "Events"),
    ("campaign", "Giving"),
];

/// These destinations alone may be serialized into public pages. Staff service
/// shortcuts remain in the authenticated workspace.
pub const PUBLIC_LINKS: &[(&str, &str)] = &[
    ("store_url", "Public store"),
    ("donation_url", "Donation checkout"),
    ("membership_url", "Membership checkout"),
    ("newsletter_url", "Newsletter signup"),
    ("volunteer_url", "Volunteer information or signup"),
];
