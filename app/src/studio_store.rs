//! Native content storage. All read/modify/write operations hold the same connection lock.
use crate::{
    cms::{Catalog, Content},
    studio_model::*,
};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use rand::{rngs::OsRng, RngCore};
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

pub type Result<T> = std::result::Result<T, String>;
pub static STORE: OnceLock<Arc<Store>> = OnceLock::new();
pub fn store() -> Result<&'static Arc<Store>> {
    STORE.get().ok_or("CMS is not enabled.".into())
}
pub fn now() -> String {
    Utc::now().to_rfc3339()
}
pub fn token() -> String {
    let mut b = [0u8; 24];
    OsRng.fill_bytes(&mut b);
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn digest(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}
pub fn hash_password(s: &str) -> Result<String> {
    if s.len() < 12 || s.len() > 256 {
        return Err("Use a password with 12–256 characters.".into());
    }
    Argon2::default()
        .hash_password(s.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}
fn db_error(e: rusqlite::Error) -> String {
    eprintln!("Trail database: {e}");
    "The database could not complete that change.".into()
}
pub struct Store {
    pub db: Mutex<Connection>,
    pub dir: PathBuf,
    pub origin: String,
    pub attempts: Mutex<HashMap<String, (u32, i64)>>,
}
fn all_items(db: &Connection) -> Result<Vec<Item>> {
    let mut s = db
        .prepare("SELECT data FROM items ORDER BY id")
        .map_err(db_error)?;
    let rows = s
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(db_error)?;
    rows.map(|r| serde_json::from_str(&r.map_err(db_error)?).map_err(|e| e.to_string()))
        .collect()
}
fn get_item(db: &Connection, id: u64) -> Result<Item> {
    let data: Option<String> = db
        .query_row("SELECT data FROM items WHERE id=?", [id as i64], |r| {
            r.get(0)
        })
        .optional()
        .map_err(db_error)?;
    serde_json::from_str(&data.ok_or("Not found.")?).map_err(|e| e.to_string())
}
fn write_item(db: &Connection, i: &Item) -> Result<()> {
    db.execute(
        "INSERT INTO items(id,data) VALUES(?,?) ON CONFLICT(id) DO UPDATE SET data=excluded.data",
        params![i.content.id as i64, serde_json::to_string(i).unwrap()],
    )
    .map_err(db_error)?;
    Ok(())
}
fn users(db: &Connection) -> Result<Vec<User>> {
    let mut s = db
        .prepare("SELECT id,login,name,role FROM users ORDER BY id")
        .map_err(db_error)?;
    let result = s
        .query_map([], |r| {
            Ok(User {
                id: r.get::<_, i64>(0)? as u64,
                login: r.get(1)?,
                name: r.get(2)?,
                role: r.get(3)?,
            })
        })
        .map_err(db_error)?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(db_error);
    result
}
pub fn valid_url(s: &str) -> bool {
    s.is_empty()
        || (!s
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
            && (s.starts_with("https://")
                || s.starts_with("http://")
                || (s.starts_with('/') && !s.starts_with("//"))))
}
fn secure_destination(value: &str) -> bool {
    if !value.starts_with("https://") || !valid_url(value) { return false; }
    value.parse::<axum::http::Uri>().is_ok_and(|uri| {
        uri.host().is_some_and(|host| !host.is_empty())
            && uri.authority().is_some_and(|authority| !authority.as_str().contains('@'))
    })
}
fn clean(d: &mut Draft) -> Result<()> {
    if d.title.len() > 720 || d.summary.len() > 8000 || d.body.len() > 100000 || d.fields.len() > 40
    {
        return Err("The content is too long.".into());
    }
    d.slug = d.slug.trim().to_owned();
    if d.slug.len() > 120
        || !d
            .slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return Err("Use lowercase letters, numbers and hyphens in the address.".into());
    }
    for (k, v) in &d.fields {
        if k.len() > 60 || v.len() > if k == crate::section_model::FIELD {65000} else {4000} {
            return Err("A field is too long.".into());
        }
        if k == crate::section_model::FIELD { crate::section_model::validate(v)?; }
        if ["image", "summer_image", "url", "button_url", "history_url"].contains(&k.as_str()) && !valid_url(v) {
            return Err(format!(
                "Use a complete http(s) address or a path beginning with / for {k}."
            ));
        }
        if ["image_x", "image_y", "summer_x", "summer_y"].contains(&k.as_str())
            && !v.is_empty() && !v.parse::<u8>().is_ok_and(|n| n <= 100) {
            return Err("Photo focus must be a number from 0 to 100.".into());
        }
        if k == "image_layout" && !["", "wide", "tall"].contains(&v.as_str()) {
            return Err("Choose a wide or tall photo layout.".into());
        }
        if k == "support_interest" && !v.is_empty() && !crate::support_model::AREAS.iter().any(|(key,_)|*key==v) {return Err("Choose a valid support interest key.".into());}
        if k == "map_stop" && !v.is_empty() && !crate::explorer::STOPS.iter().any(|(id,_)|*id==v) {
            return Err("Choose a verified place on the trail map.".into());
        }
        if ["essay_photo_1","essay_photo_2"].contains(&k.as_str()) && !v.is_empty() && v!="none" && crate::photography::named(v).is_none() {
            return Err("Choose a supporting photograph from the credited collection.".into());
        }
        if k=="story_map" && !["","none","whole-trail"].contains(&v.as_str()) && !crate::explorer::STOPS.iter().any(|(id,_)|*id==v) {
            return Err("Choose a verified supporting map.".into());
        }
    }
    Ok(())
}
fn date(s: &str) -> Option<DateTime<chrono::FixedOffset>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .or_else(|| DateTime::parse_from_rfc3339(&format!("{s}T00:00:00Z")).ok())
}
fn ready(d: &Draft, kind: &str) -> Result<()> {
    let sections=crate::section_model::parse(&d.field(crate::section_model::FIELD))?;
    let issues=crate::section_model::issues(&sections);
    if !issues.is_empty(){return Err(issues.join(" "));}
    if d.title.trim().is_empty() || d.slug.is_empty() {
        return Err("Add a headline and web address before publishing.".into());
    }
    if !d.field("image").is_empty() && d.field("image_alt").trim().is_empty() {
        return Err("Describe the main photo before publishing.".into());
    }
    if kind == "resource" && d.field("url").is_empty() {
        return Err("Add a document or resource link.".into());
    }
    if !d.field("button_text").is_empty() && d.field("button_url").is_empty() {
        return Err("Add the button destination.".into());
    }
    if kind == "event" {
        let start = date(&d.field("start")).ok_or("Add a valid event start date.")?;
        if !d.field("end").is_empty() && date(&d.field("end")).is_none_or(|e| e < start) {
            return Err("Event end must be on or after the start.".into());
        }
    }
    Ok(())
}
fn revision(i: &mut Item, d: Draft, label: &str, by: &str) {
    i.history.insert(
        0,
        Revision {
            id: token(),
            at: now(),
            by: by.into(),
            label: label.into(),
            content: d,
        },
    );
    i.history.truncate(75);
}
impl Store {
    pub fn open(dir: PathBuf, origin: String) -> Result<Self> {
        std::fs::create_dir_all(dir.join("uploads")).map_err(|e| e.to_string())?;
        let db = Connection::open(dir.join("trail.sqlite3")).map_err(db_error)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=5000;
 CREATE TABLE IF NOT EXISTS items(id INTEGER PRIMARY KEY,data TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS users(id INTEGER PRIMARY KEY,login TEXT NOT NULL UNIQUE,name TEXT NOT NULL,role TEXT NOT NULL,password TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS sessions(token TEXT PRIMARY KEY,user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,csrf TEXT NOT NULL,expires INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS previews(token TEXT PRIMARY KEY,item_id INTEGER NOT NULL,content TEXT NOT NULL,expires INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS media(id TEXT PRIMARY KEY,data TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL);").map_err(db_error)?;
        crate::support_store::init(&db)?;
        let s = Self {
            db: Mutex::new(db),
            dir,
            origin,
            attempts: Mutex::new(HashMap::new()),
        };
        s.bootstrap()?;
        Ok(s)
    }
    fn bootstrap(&self) -> Result<()> {
        let mut db = self.db.lock().unwrap();
        let count: i64 = db
            .query_row("SELECT count(*) FROM users", [], |r| r.get(0))
            .map_err(db_error)?;
        if count > 0 {
            return Ok(());
        }
        let login = std::env::var("TRAIL_ADMIN_USER").unwrap_or("trail-admin".into());
        let password = std::env::var("TRAIL_ADMIN_PASSWORD")
            .map_err(|_| "Set TRAIL_ADMIN_PASSWORD for the first startup.".to_string())?;
        let hash = hash_password(&password)?;
        let tx = db.transaction().map_err(db_error)?;
        let mut imported = if self.dir.join("import.json").exists() {
            let raw =
                std::fs::read_to_string(self.dir.join("import.json")).map_err(|e| e.to_string())?;
            serde_json::from_str::<serde_json::Value>(
                &raw.replace(
                    "http://localhost:8090/wp-content/uploads/",
                    "/media/legacy/",
                )
                .replace("/trail-assets/", "/"),
            )
            .map_err(|e| e.to_string())?
        } else {
            serde_json::json!({})
        };
        let user = imported["users"]
            .as_array()
            .and_then(|a| a.first())
            .cloned()
            .and_then(|v| serde_json::from_value::<User>(v).ok())
            .unwrap_or(User {
                id: 1,
                login: login.clone(),
                name: "Trail Administrator".into(),
                role: "admin".into(),
            });
        tx.execute(
            "INSERT INTO users(id,login,name,role,password) VALUES(?,?,?,'admin',?)",
            params![user.id as i64, login, user.name, hash],
        )
        .map_err(db_error)?;
        let items: Vec<Item> = if imported["items"].is_array() {
            serde_json::from_value(imported["items"].take()).map_err(|e| e.to_string())?
        } else {
            #[derive(serde::Deserialize)]
            struct Seed {
                #[serde(flatten)]
                content: Content,
                #[serde(default)]
                status: String,
            }
            let seeds: Vec<Seed> = serde_json::from_str(include_str!("../cms/seed.json"))
                .map_err(|e| format!("Invalid seed content: {e}"))?;
            seeds
                .into_iter()
                .enumerate()
                .map(|(n, seed)| {
                    let published = seed.status != "draft";
                    let mut c = seed.content;
                    c.id = n as u64 + 1;
                    c.path = content_path(&c.kind, &c.slug);
                    Item {
                        draft: if published {
                            None
                        } else {
                            Some(Draft::from_content(&c))
                        },
                        content: c,
                        status: if published { "publish" } else { "draft" }.into(),
                        modified: now(),
                        schedule: None,
                        version: 1,
                        author: user.name.clone(),
                        owner: user.id,
                        ever_published: published,
                        history: vec![],
                    }
                })
                .collect()
        };
        // The seed omits runtime IDs and paths; accept it through the same DTO with defaults.
        for i in &items {
            write_item(&tx, i)?;
        }
        let settings = if imported["connections"].is_object() {
            imported["connections"].clone()
        } else {
            serde_json::json!({})
        };
        tx.execute(
            "INSERT INTO settings(key,value) VALUES('connections',?)",
            [settings.to_string()],
        )
        .map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(())
    }
    pub fn login(&self, login: &str, password: &str, peer: &str) -> Result<(Session, String)> {
        let time = Utc::now().timestamp();
        {
            let mut a = self.attempts.lock().unwrap();
            a.retain(|_, v| v.1 > time - 900);
            if a.len() > 2000 {
                return Err("Please wait before trying again.".into());
            }
            let entry = a.entry(peer.into()).or_insert((0, time));
            if entry.0 >= 10 {
                return Err("Too many attempts. Try again in 15 minutes.".into());
            }
            entry.0 += 1;
        }
        let found = {
            let db = self.db.lock().unwrap();
            db.query_row(
                "SELECT id,login,name,role,password FROM users WHERE login=?",
                [login],
                |r| {
                    Ok((
                        User {
                            id: r.get::<_, i64>(0)? as u64,
                            login: r.get(1)?,
                            name: r.get(2)?,
                            role: r.get(3)?,
                        },
                        r.get::<_, String>(4)?,
                    ))
                },
            )
            .optional()
            .map_err(db_error)?
        };
        let (user, hash) = found.ok_or("The username or password is incorrect.")?;
        if password.len() > 256
            || Argon2::default()
                .verify_password(
                    password.as_bytes(),
                    &PasswordHash::new(&hash).map_err(|e| e.to_string())?,
                )
                .is_err()
        {
            return Err("The username or password is incorrect.".into());
        }
        let raw = token();
        let session = Session {
            user,
            csrf: token(),
        };
        self.db
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO sessions(token,user_id,csrf,expires) VALUES(?,?,?,?)",
                params![
                    digest(&raw),
                    session.user.id as i64,
                    session.csrf,
                    time + 43200
                ],
            )
            .map_err(db_error)?;
        self.attempts.lock().unwrap().remove(peer);
        Ok((session, raw))
    }
    pub fn session(&self, raw: &str) -> Result<Session> {
        self.db.lock().unwrap().query_row("SELECT u.id,u.login,u.name,u.role,s.csrf FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token=? AND s.expires>?",params![digest(raw),Utc::now().timestamp()],|r|Ok(Session{user:User{id:r.get::<_,i64>(0)? as u64,login:r.get(1)?,name:r.get(2)?,role:r.get(3)?},csrf:r.get(4)?})).optional().map_err(db_error)?.ok_or("Sign in to continue.".into())
    }
    pub fn logout(&self, raw: &str) -> Result<()> {
        self.db
            .lock()
            .unwrap()
            .execute("DELETE FROM sessions WHERE token=?", [digest(raw)])
            .map_err(db_error)?;
        Ok(())
    }
    pub fn workspace(&self, u: &User) -> Result<Workspace> {
        let db = self.db.lock().unwrap();
        let items = all_items(&db)?
            .into_iter()
            .filter(|i| u.can_edit(i))
            .collect();
        let mut stmt = db
            .prepare("SELECT data FROM media ORDER BY rowid DESC")
            .map_err(db_error)?;
        let media = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(db_error)?
            .map(|x| serde_json::from_str(&x.map_err(db_error)?).map_err(|e| e.to_string()))
            .collect::<Result<Vec<Media>>>()?;
        let connections = serde_json::from_str(
            &db.query_row(
                "SELECT value FROM settings WHERE key='connections'",
                [],
                |r| r.get::<_, String>(0),
            )
            .map_err(db_error)?,
        )
        .map_err(|e| e.to_string())?;
        Ok(Workspace {
            items,
            media,
            connections,
            users: if u.role == "admin" {
                users(&db)?
            } else {
                vec![u.clone()]
            },
        })
    }
    pub fn create(&self, u: &User, kind: &str) -> Result<Item> {
        if !KINDS.iter().any(|(k, _)| *k == kind) {
            return Err("Unknown content type.".into());
        }
        if kind == "page" && !u.publisher() {
            return Err("Only editors can add pages.".into());
        }
        let db = self.db.lock().unwrap();
        let id: u64 = db
            .query_row("SELECT coalesce(max(id),0)+1 FROM items", [], |r| {
                r.get::<_, i64>(0).map(|n| n as u64)
            })
            .map_err(db_error)?;
        let i = Item {
            content: Content {
                id,
                kind: kind.into(),
                ..Default::default()
            },
            status: "draft".into(),
            modified: now(),
            draft: Some(Draft::default()),
            schedule: None,
            version: 1,
            author: u.name.clone(),
            owner: u.id,
            ever_published: false,
            history: vec![],
        };
        write_item(&db, &i)?;
        Ok(i)
    }
    pub fn change(&self, u: &User, id: u64, c: Change) -> Result<Item> {
        let db = self.db.lock().unwrap();
        let mut i = get_item(&db, id)?;
        if !u.can_edit(&i) {
            return Err("You cannot edit this content.".into());
        }
        if i.version != c.version {
            return Err(
                "Conflict: someone saved a newer version. Reload before saving your changes."
                    .into(),
            );
        }
        let mut d = c.content.unwrap_or_else(|| i.editable());
        clean(&mut d)?;
        if i.schedule
            .as_ref()
            .is_some_and(|s| s.content.slug != d.slug)
        {
            return Err("Cancel the schedule before changing its address.".into());
        }
        if i.ever_published && d.slug != i.content.slug {
            return Err("Published addresses are permanent. Keep the original address.".into());
        }
        if !d.slug.is_empty()
            && all_items(&db)?.iter().any(|other| {
                other.content.id != id
                    && (other.content.kind == i.content.kind
                        && [
                            other.content.slug.clone(),
                            other.editable().slug,
                            other
                                .schedule
                                .as_ref()
                                .map(|s| s.content.slug.clone())
                                .unwrap_or_default(),
                        ]
                        .contains(&d.slug))
            })
        {
            return Err("That web address is already in use.".into());
        }
        if i.content.kind == "page"
            && !(i.ever_published && i.content.slug == "stories" && d.slug == "stories")
            && [
                "admin",
                "api",
                "media",
                "cms-preview",
                "support",
                "places",
                "stories",
                "events",
                "campaigns",
                "preview",
                "credits",
            ]
            .contains(&d.slug.as_str())
        {
            return Err("That address is reserved.".into());
        }
        match c.action.as_str() {
            "save" => {
                i.draft = Some(d);
            }
            "submit" => {
                i.draft = Some(d);
                if i.status != "publish" {
                    i.status = "pending".into();
                }
            }
            "publish" | "schedule" => {
                if !u.publisher() {
                    return Err("Only an editor can publish or schedule content.".into());
                }
                ready(&d, &i.content.kind)?;
                let map_stop=d.field("map_stop");
                if i.content.kind=="place" && !map_stop.is_empty() && all_items(&db)?.iter().any(|other|other.content.id!=id && other.content.kind=="place" && ((other.status=="publish" && other.content.field("map_stop")==map_stop) || other.schedule.as_ref().is_some_and(|s|s.content.field("map_stop")==map_stop))) {
                    return Err("Another published or scheduled place already uses this map location. Remove that connection first.".into());
                }
                if c.action == "schedule" {
                    let at = c.at.ok_or("Choose a publication time.")?;
                    let dt = DateTime::parse_from_rfc3339(&at)
                        .map_err(|_| "Choose a valid publication time.")?;
                    if dt <= Utc::now() {
                        return Err("Choose a future publication time.".into());
                    }
                    i.schedule = Some(Schedule {
                        at,
                        content: d.clone(),
                    });
                    i.draft = Some(d);
                } else {
                    if i.ever_published {
                        let old = Draft::from_content(&i.content);
                        revision(&mut i, old, "Before publishing", &u.name);
                    }
                    d.apply(&mut i.content);
                    i.status = "publish".into();
                    i.ever_published = true;
                    i.draft = None;
                    i.schedule = None;
                }
            }
            "restore" => {
                let r = i
                    .history
                    .iter()
                    .find(|r| Some(&r.id) == c.revision.as_ref())
                    .ok_or("Revision not found.")?;
                let mut restored = r.content.clone();
                if i.ever_published {
                    restored.slug = i.content.slug.clone();
                }
                i.draft = Some(restored);
            }
            "discard" => {
                if let Some(draft) = i.draft.clone() {
                    revision(&mut i, draft, "Discarded draft", &u.name);
                }
                i.draft = None;
            }
            "cancel-schedule" => {
                if !u.publisher() {
                    return Err("Only an editor can change a schedule.".into());
                }
                i.schedule = None;
            }
            "archive" => {
                if !u.publisher() {
                    return Err("Only an editor can archive content.".into());
                }
                if i.content.kind == "page"
                    && [
                        "home",
                        "about",
                        "explore",
                        "stories",
                        "resources",
                        "get-involved",
                        "store",
                    ]
                    .contains(&i.content.slug.as_str())
                {
                    return Err("Core pages cannot be archived.".into());
                }
                i.status = "private".into();
                i.schedule = None;
            }
            _ => return Err("Unknown action.".into()),
        }
        i.version += 1;
        i.modified = now();
        write_item(&db, &i)?;
        Ok(i)
    }
    pub fn preview(&self, u: &User, id: u64) -> Result<String> {
        let db = self.db.lock().unwrap();
        let i = get_item(&db, id)?;
        if !u.can_edit(&i) {
            return Err("You cannot preview this content.".into());
        }
        let mut content = i.content.clone();
        i.editable().apply(&mut content);
        let raw = token();
        db.execute(
            "INSERT INTO previews(token,item_id,content,expires) VALUES(?,?,?,?)",
            params![
                digest(&raw),
                id as i64,
                serde_json::to_string(&content).unwrap(),
                Utc::now().timestamp() + 1800
            ],
        )
        .map_err(db_error)?;
        Ok(format!("/cms-preview/{raw}"))
    }
    pub fn public(&self, preview: Option<String>) -> Result<Catalog> {
        self.tick()?;
        let db = self.db.lock().unwrap();
        let mut items: Vec<Content> = all_items(&db)?
            .into_iter()
            .filter(|i| i.status == "publish")
            .map(|i| crate::section_model::public_content(i.content))
            .filter(|i| i.kind != "event" || date(&i.field("end")).is_none_or(|e| e >= Utc::now()))
            .collect();
        let mut preview_id = None;
        if let Some(raw) = preview {
            let content: Option<String> = db
                .query_row(
                    "SELECT content FROM previews WHERE token=? AND expires>?",
                    params![digest(&raw), Utc::now().timestamp()],
                    |r| r.get(0),
                )
                .optional()
                .map_err(db_error)?;
            let p: Content = serde_json::from_str(
                &content.ok_or("This preview has expired. Open a new preview in Trail Studio.")?,
            )
            .map_err(|e| e.to_string())?;
            preview_id = Some(p.id);
            items.retain(|i| i.id != p.id);
            items.push(p);
        }
        let connections: HashMap<String, String> = serde_json::from_str(
            &db.query_row("SELECT value FROM settings WHERE key='connections'", [], |r| r.get::<_, String>(0)).map_err(db_error)?
        ).map_err(|e| e.to_string())?;
        let public_links = connections.into_iter().filter(|(key, value)| {
            PUBLIC_LINKS.iter().any(|(allowed, _)| key == allowed)
                && !value.is_empty() && secure_destination(value)
        }).collect();
        Ok(Catalog {
            items,
            preview_id,
            download_base: "/api/cms/download/".into(),
            public_links,
        })
    }
    pub fn tick(&self) -> Result<()> {
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(db_error)?;
        for mut i in all_items(&tx)? {
            if let Some(s) = i
                .schedule
                .clone()
                .filter(|s| date(&s.at).is_some_and(|at| at <= Utc::now()))
            {
                if i.ever_published {
                    let old = Draft::from_content(&i.content);
                    revision(&mut i, old, "Before scheduled publishing", "Scheduler");
                }
                if i.draft.as_ref() == Some(&s.content) {
                    i.draft = None;
                }
                s.content.apply(&mut i.content);
                i.schedule = None;
                i.status = "publish".into();
                i.ever_published = true;
                i.version += 1;
                i.modified = now();
                write_item(&tx, &i)?;
            }
        }
        tx.execute(
            "DELETE FROM sessions WHERE expires<?",
            [Utc::now().timestamp()],
        )
        .map_err(db_error)?;
        tx.execute(
            "DELETE FROM previews WHERE expires<?",
            [Utc::now().timestamp()],
        )
        .map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(())
    }
    pub fn connections(&self, u: &User, data: HashMap<String, String>) -> Result<()> {
        if u.role != "admin" {
            return Err("Only administrators can change connections.".into());
        }
        if data.len() > 8
            || data.iter().any(|(k, v)| {
                !(["square", "donors", "email"].contains(&k.as_str()) || PUBLIC_LINKS.iter().any(|(allowed, _)| k == allowed))
                    || (!v.is_empty() && !secure_destination(v))
                    || v.len() > 2000
            })
        {
            return Err("Use a complete HTTPS address for each connection, or leave it blank.".into());
        }
        self.db
            .lock()
            .unwrap()
            .execute(
                "UPDATE settings SET value=? WHERE key='connections'",
                [serde_json::to_string(&data).unwrap()],
            )
            .map_err(db_error)?;
        Ok(())
    }
    pub fn add_user(
        &self,
        u: &User,
        login: &str,
        name: &str,
        role: &str,
        password: &str,
    ) -> Result<()> {
        if u.role != "admin" {
            return Err("Only administrators can manage staff.".into());
        }
        if login.is_empty()
            || login.len() > 60
            || !login
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || name.trim().is_empty()
            || name.len() > 120
            || !["admin", "editor", "contributor"].contains(&role)
        {
            return Err("Enter a name, username and valid role.".into());
        }
        let hash = hash_password(password)?;
        self.db
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO users(login,name,role,password) VALUES(?,?,?,?)",
                params![login, name, role, hash],
            )
            .map_err(|_| "That username is already in use.".to_string())?;
        Ok(())
    }
    pub fn password(&self, u: &User, id: u64, password: &str) -> Result<()> {
        if u.role != "admin" && u.id != id {
            return Err("Only administrators can reset another person's password.".into());
        }
        let hash = hash_password(password)?;
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(db_error)?;
        if tx
            .execute(
                "UPDATE users SET password=? WHERE id=?",
                params![hash, id as i64],
            )
            .map_err(db_error)?
            == 0
        {
            return Err("User not found.".into());
        }
        tx.execute("DELETE FROM sessions WHERE user_id=?", [id as i64])
            .map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(())
    }
    pub fn remove_user(&self, u: &User, id: u64) -> Result<()> {
        if u.role != "admin" || u.id == id {
            return Err("You cannot remove this account.".into());
        }
        let mut db = self.db.lock().unwrap();
        let tx = db.transaction().map_err(db_error)?;
        for mut i in all_items(&tx)? {
            if i.owner == id {
                i.owner = u.id;
                write_item(&tx, &i)?
            }
        }
        tx.execute("DELETE FROM users WHERE id=?", [id as i64])
            .map_err(db_error)?;
        tx.commit().map_err(db_error)?;
        Ok(())
    }
    pub fn add_media(&self, m: &Media) -> Result<()> {
        self.db
            .lock()
            .unwrap()
            .execute(
                "INSERT INTO media(id,data) VALUES(?,?)",
                params![m.id, serde_json::to_string(m).unwrap()],
            )
            .map_err(db_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static ENV_LOCK: Mutex<()> = Mutex::new(());
    fn setup() -> (std::sync::MutexGuard<'static, ()>, Store, User) {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("TRAIL_ADMIN_PASSWORD", "native-test-password-123");
        let dir = std::env::temp_dir().join(format!("trail-studio-test-{}", token()));
        let store = Store::open(dir, "http://localhost:3110".into()).unwrap();
        let user = store
            .login("trail-admin", "native-test-password-123", "test")
            .unwrap()
            .0
            .user;
        (lock, store, user)
    }
    fn draft(title: &str) -> Draft {
        Draft {
            title: title.into(),
            slug: "test-story".into(),
            summary: "A useful introduction".into(),
            body: "Original text".into(),
            ..Default::default()
        }
    }
    fn change(version: u64, action: &str, content: Draft) -> Change {
        Change {
            version,
            action: action.into(),
            content: Some(content),
            at: None,
            revision: None,
        }
    }
    #[test]
    fn seeded_story_index_can_be_edited_but_new_pages_cannot_claim_reserved_routes() {
        let (_lock,s,u)=setup();
        let existing={let db=s.db.lock().unwrap();all_items(&db).unwrap().into_iter().find(|i|i.content.kind=="page"&&i.content.slug=="stories").unwrap()};
        let mut content=existing.editable();content.fields.insert("image".into(),"/images/dawn.webp".into());content.fields.insert("image_alt".into(),"Winter light".into());
        let updated=s.change(&u,existing.content.id,change(existing.version,"publish",content)).unwrap();assert_eq!(updated.content.field("image"),"/images/dawn.webp");
        for slug in ["admin","api","places","stories","credits"] {
            let page=s.create(&u,"page").unwrap();let mut content=draft("Reserved route check");content.slug=slug.into();
            assert!(s.change(&u,page.content.id,change(page.version,"publish",content)).is_err(),"new page claimed {slug}");
        }
    }
    #[test]
    fn explorer_overrides_are_private_until_published_and_locations_are_unique() {
        let (_lock,s,u)=setup();let first=s.create(&u,"place").unwrap();
        let mut d=draft("Nome field guide");d.slug="nome-field-guide".into();d.fields.insert("map_stop".into(),"nome".into());d.fields.insert("trail_history".into(),"Reviewed history".into());
        let saved=s.change(&u,first.content.id,change(first.version,"save",d.clone())).unwrap();
        assert!(!s.public(None).unwrap().items.iter().any(|p|p.field("map_stop")=="nome"));
        let published=s.change(&u,first.content.id,change(saved.version,"publish",d.clone())).unwrap();
        let stops=crate::explorer::stops_with_content(s.public(None).unwrap().items);
        assert_eq!(stops[5].history,"Reviewed history");
        let second=s.create(&u,"place").unwrap();d.slug="second-nome-guide".into();
        assert!(s.change(&u,second.content.id,change(second.version,"publish",d.clone())).unwrap_err().contains("already uses this map location"));
        d.fields.insert("map_stop".into(),"invented-location".into());
        assert!(s.change(&u,second.content.id,change(second.version,"save",d)).unwrap_err().contains("verified place"));
        assert_eq!(published.content.field("map_stop"),"nome");
    }
    #[test]
    fn photo_presentation_is_validated_and_previewed_without_publishing() {
        let (_lock, s, u) = setup();
        let item = s.create(&u, "story").unwrap();
        let mut d = draft("A photograph");
        d.fields.insert("image_x".into(), "22".into());
        d.fields.insert("image_y".into(), "78".into());
        d.fields.insert("image_layout".into(), "tall".into());
        let saved = s.change(&u, item.content.id, change(item.version, "save", d.clone())).unwrap();
        assert_eq!(saved.editable().field("image_y"), "78");
        assert!(!s.public(None).unwrap().items.iter().any(|i| i.id == item.content.id));
        for (key, value) in [("image_x", "101"), ("image_y", "-1"), ("summer_x", "left"), ("image_layout", "arbitrary-css")] {
            let mut invalid = d.clone();invalid.fields.insert(key.into(), value.into());
            assert!(s.change(&u, item.content.id, change(saved.version, "save", invalid)).is_err());
        }
        let published = s.change(&u, item.content.id, change(saved.version, "publish", d)).unwrap();
        assert_eq!(published.content.image_position(), "object-position:22% 78%");
    }
    #[test]
    fn sections_are_private_validated_and_kept_in_revisions() {
        use crate::section_model::{Section,Sections,FIELD};
        let (_lock,s,u)=setup();let item=s.create(&u,"page").unwrap();let mut d=draft("Approved sections");
        let mut section=Section::new(1,"split");let encode=|s:Section|serde_json::to_string(&Sections{version:1,sections:vec![s]}).unwrap();
        d.fields.insert(FIELD.into(),encode(section.clone()));let saved=s.change(&u,item.content.id,change(item.version,"save",d.clone())).unwrap();
        assert!(s.change(&u,item.content.id,change(saved.version,"publish",d.clone())).unwrap_err().contains("add a heading"));
        assert!(!s.public(None).unwrap().items.iter().any(|i|i.id==item.content.id));
        section.title="A place in the story".into();section.body="Reviewed content".into();section.image="/images/dawn.webp".into();section.alt="Winter light on the trail".into();section.caption="Winter landscape".into();section.credit="Bob Wick / BLM".into();section.x=25;section.y=70;
        d.fields.insert(FIELD.into(),encode(section.clone()));let published=s.change(&u,item.content.id,change(saved.version,"publish",d.clone())).unwrap();
        section.title="Private next chapter".into();d.fields.insert(FIELD.into(),encode(section));let saved=s.change(&u,item.content.id,change(published.version,"save",d.clone())).unwrap();
        let token=s.preview(&u,item.content.id).unwrap();let preview=s.public(Some(token.trim_start_matches("/cms-preview/").to_string())).unwrap();assert!(preview.items.iter().find(|i|i.id==item.content.id).unwrap().field(FIELD).contains("Private next chapter"));
        assert!(!s.public(None).unwrap().items.iter().find(|i|i.id==item.content.id).unwrap().field(FIELD).contains("Private next chapter"));
        let updated=s.change(&u,item.content.id,change(saved.version,"publish",d)).unwrap();assert!(updated.history[0].content.field(FIELD).contains("A place in the story"));
    }
    #[test]
    fn public_destinations_are_opt_in_validated_and_separate_from_staff_links() {
        let (_lock, s, u) = setup();
        assert!(s.public(None).unwrap().public_links.is_empty());
        let settings = HashMap::from([
            ("square".into(), "https://squareup.com/dashboard".into()),
            ("donors".into(), "https://example.org/private-records".into()),
            ("store_url".into(), "https://example.org/shop".into()),
            ("newsletter_url".into(), String::new()),
        ]);
        s.connections(&u, settings).unwrap();
        assert_eq!(s.public(None).unwrap().public_links, HashMap::from([("store_url".into(), "https://example.org/shop".into())]));
        for unsafe_url in ["http://example.org", "javascript:alert(1)", "https://", "https://user:password@example.org", "https://example.org/a b"] {
            assert!(s.connections(&u, HashMap::from([("store_url".into(), unsafe_url.into())])).is_err());
        }
        let editor=User { role: "editor".into(), ..u.clone() };
        assert!(s.connections(&editor, HashMap::new()).is_err());
        assert_eq!(s.public(None).unwrap().public_links.len(), 1);
        s.connections(&u, HashMap::new()).unwrap();
        assert!(s.public(None).unwrap().public_links.is_empty());
    }
    #[test]
    fn draft_publish_revision_and_stable_slug() {
        let (_lock, s, u) = setup();
        assert_eq!(s.public(None).unwrap().items.len(), 24);
        let i = s.create(&u, "story").unwrap();
        let i = s
            .change(
                &u,
                i.content.id,
                change(i.version, "save", draft("First draft")),
            )
            .unwrap();
        assert!(!s
            .public(None)
            .unwrap()
            .items
            .iter()
            .any(|x| x.id == i.content.id));
        let i = s
            .change(&u, i.content.id, change(i.version, "publish", i.editable()))
            .unwrap();
        let i = s
            .change(
                &u,
                i.content.id,
                change(i.version, "save", draft("Second draft")),
            )
            .unwrap();
        assert_eq!(
            s.public(None)
                .unwrap()
                .items
                .iter()
                .find(|x| x.id == i.content.id)
                .unwrap()
                .title,
            "First draft"
        );
        let mut moved = i.editable();
        moved.slug = "changed".into();
        assert!(s
            .change(&u, i.content.id, change(i.version, "publish", moved))
            .unwrap_err()
            .contains("permanent"));
        let i = s
            .change(&u, i.content.id, change(i.version, "publish", i.editable()))
            .unwrap();
        assert_eq!(i.history[0].content.title, "First draft");
        let mut c = change(i.version, "restore", i.editable());
        c.revision = Some(i.history[0].id.clone());
        let i = s.change(&u, i.content.id, c).unwrap();
        assert_eq!(i.editable().title, "First draft");
        assert_eq!(i.content.title, "Second draft");
    }
    #[test]
    fn conflict_and_duplicate_slug_are_rejected() {
        let (_lock, s, u) = setup();
        let a = s.create(&u, "story").unwrap();
        let b = s.create(&u, "story").unwrap();
        s.change(&u, a.content.id, change(a.version, "save", draft("A")))
            .unwrap();
        assert!(s
            .change(
                &u,
                a.content.id,
                change(a.version, "save", draft("Outdated"))
            )
            .unwrap_err()
            .starts_with("Conflict:"));
        assert!(s
            .change(&u, b.content.id, change(b.version, "save", draft("B")))
            .unwrap_err()
            .contains("already in use"));
    }
    #[test]
    fn roles_and_revoked_sessions() {
        let (_lock, s, u) = setup();
        s.add_user(
            &u,
            "writer",
            "Trail Writer",
            "contributor",
            "another-secure-password",
        )
        .unwrap();
        let (session, cookie) = s
            .login("writer", "another-secure-password", "writer-ip")
            .unwrap();
        let foreign = s.create(&u, "story").unwrap();
        assert!(s
            .change(
                &session.user,
                foreign.content.id,
                change(foreign.version, "save", draft("No"))
            )
            .is_err());
        let own = s.create(&session.user, "story").unwrap();
        assert!(s
            .change(
                &session.user,
                own.content.id,
                change(own.version, "publish", draft("No"))
            )
            .is_err());
        let own = s
            .change(
                &session.user,
                own.content.id,
                change(own.version, "submit", draft("For review")),
            )
            .unwrap();
        assert_eq!(own.status, "pending");
        assert!(s.remove_user(&u, u.id).is_err());
        s.remove_user(&u, session.user.id).unwrap();
        assert!(s.session(&cookie).is_err());
        assert_eq!(
            s.workspace(&u)
                .unwrap()
                .items
                .iter()
                .find(|i| i.content.id == own.content.id)
                .unwrap()
                .owner,
            u.id
        );
    }
    #[test]
    fn preview_is_frozen_and_expires() {
        let (_lock, s, u) = setup();
        let i = s.create(&u, "story").unwrap();
        let i = s
            .change(
                &u,
                i.content.id,
                change(i.version, "save", draft("Preview one")),
            )
            .unwrap();
        let path = s.preview(&u, i.content.id).unwrap();
        let token = path.rsplit('/').next().unwrap().to_string();
        s.change(
            &u,
            i.content.id,
            change(i.version, "save", draft("Preview two")),
        )
        .unwrap();
        assert_eq!(
            s.public(Some(token.clone()))
                .unwrap()
                .items
                .iter()
                .find(|p| p.id == i.content.id)
                .unwrap()
                .title,
            "Preview one"
        );
        s.db.lock()
            .unwrap()
            .execute("UPDATE previews SET expires=0", [])
            .unwrap();
        assert!(s.public(Some(token)).is_err());
    }
    #[test]
    fn schedule_keeps_later_edits_private() {
        let (_lock, s, u) = setup();
        let i = s.create(&u, "story").unwrap();
        let mut c = change(i.version, "schedule", draft("Scheduled snapshot"));
        c.at = Some((Utc::now() + chrono::Duration::hours(1)).to_rfc3339());
        let i = s.change(&u, i.content.id, c).unwrap();
        let mut i = s
            .change(
                &u,
                i.content.id,
                change(i.version, "save", draft("Later private draft")),
            )
            .unwrap();
        i.schedule.as_mut().unwrap().at = (Utc::now() - chrono::Duration::seconds(2)).to_rfc3339();
        write_item(&s.db.lock().unwrap(), &i).unwrap();
        s.tick().unwrap();
        let i = get_item(&s.db.lock().unwrap(), i.content.id).unwrap();
        assert_eq!(i.content.title, "Scheduled snapshot");
        assert_eq!(i.draft.unwrap().title, "Later private draft");
        assert!(i.schedule.is_none());
    }
    #[test]
    fn validation_and_database_persistence() {
        let (_lock, s, u) = setup();
        let i = s.create(&u, "story").unwrap();
        let mut d = draft("Photo");
        d.fields
            .insert("image".into(), "javascript:alert(1)".into());
        assert!(s
            .change(&u, i.content.id, change(i.version, "save", d.clone()))
            .is_err());
        d.fields.insert("image".into(), "/images/dawn.webp".into());
        assert!(s
            .change(&u, i.content.id, change(i.version, "publish", d.clone()))
            .is_err());
        d.fields
            .insert("image_alt".into(), "Winter landscape".into());
        let i = s
            .change(&u, i.content.id, change(i.version, "publish", d))
            .unwrap();
        let dir = s.dir.clone();
        drop(s);
        let s = Store::open(dir, "http://localhost:3110".into()).unwrap();
        assert!(s
            .public(None)
            .unwrap()
            .items
            .iter()
            .any(|p| p.id == i.content.id));
    }
}
