use crate::{
    studio_model::*,
    studio_store::{self as db, store},
};
use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Multipart, Path},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, net::SocketAddr};
type ApiResult = Result<Response, ApiError>;
pub struct ApiError(String);
impl From<String> for ApiError {
    fn from(s: String) -> Self {
        Self(s)
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = if self.0.starts_with("Conflict:") {
            StatusCode::CONFLICT
        } else if self.0 == "Sign in to continue." {
            StatusCode::UNAUTHORIZED
        } else if self.0.contains("Only ")
            || self.0.contains("cannot")
            || self.0.contains("security check")
        {
            StatusCode::FORBIDDEN
        } else if self.0 == "Not found." {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(json!({"error":self.0}))).into_response()
    }
}
fn cookie(h: &HeaderMap) -> String {
    h.get("cookie")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|part| part.trim().strip_prefix("trail_session="))
        .unwrap_or("")
        .to_owned()
}
fn origin(h: &HeaderMap) -> Result<(), ApiError> {
    if h.get("origin").and_then(|v| v.to_str().ok()) != Some(store()?.origin.as_str()) {
        return Err(ApiError(
            "The request failed its security check. Reload this page.".into(),
        ));
    }
    Ok(())
}
fn auth(h: &HeaderMap, write: bool) -> Result<Session, ApiError> {
    let s = store()?.session(&cookie(h))?;
    if write {
        origin(h)?;
        let csrf = h
            .get("x-csrf-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if db::digest(csrf) != db::digest(&s.csrf) {
            return Err(ApiError(
                "The request failed its security check. Reload this page.".into(),
            ));
        }
    }
    Ok(s)
}
fn ok<T: serde::Serialize>(data: T) -> ApiResult {
    Ok(Json(data).into_response())
}
pub fn routes() -> Router<leptos::prelude::LeptosOptions> {
    Router::new()
        .route("/api/cms/session", get(session))
        .route("/api/cms/login", post(login))
        .route("/api/cms/logout", post(logout))
        .route("/api/cms/workspace", get(workspace))
        .route("/api/cms/items", post(create))
        .route("/api/cms/items/{id}", post(change))
        .route("/api/cms/preview/{id}", post(preview))
        .route("/api/cms/public", get(public))
        .route("/api/cms/download/{id}", get(download))
        .route("/wp-json/trail/v1/download/{id}", get(download))
        .route("/api/cms/connections", post(connections))
        .route("/api/cms/users", post(add_user))
        .route("/api/cms/users/{id}", post(update_user))
        .route("/api/cms/export", get(export))
        .route(
            "/api/cms/media",
            post(upload).layer(DefaultBodyLimit::max(21 * 1024 * 1024)),
        )
        .route("/media/{*path}", get(media_file))
}
async fn session(h: HeaderMap) -> ApiResult {
    ok(auth(&h, false)?)
}
#[derive(Deserialize)]
struct Login {
    login: String,
    password: String,
}
// Forwarded addresses are accepted only from the explicitly configured loopback proxy.
fn login_client(h: &HeaderMap, peer: SocketAddr, trust_proxy: bool) -> String {
    if trust_proxy && peer.ip().is_loopback() {
        if let Some(ip) = h.get("x-forwarded-for").and_then(|h|h.to_str().ok())
            .and_then(|value|value.rsplit(',').next())
            .and_then(|value|value.trim().parse::<std::net::IpAddr>().ok()) {
            return ip.to_string();
        }
    }
    peer.ip().to_string()
}
async fn login(
    h: HeaderMap,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(input): Json<Login>,
) -> ApiResult {
    origin(&h)?;
    let s = store()?.clone();
    let client = login_client(&h, peer, std::env::var("TRAIL_TRUST_PROXY").as_deref()==Ok("1"));
    let result = tokio::task::spawn_blocking(move || {
        s.login(&input.login, &input.password, &client)
    })
    .await
    .map_err(|_| ApiError("Sign in is temporarily unavailable.".into()))??;
    let secure = if store()?.origin.starts_with("https://") {
        "; Secure"
    } else {
        ""
    };
    let mut response = Json(result.0).into_response();
    response.headers_mut().insert(
        "set-cookie",
        format!(
            "trail_session={}; HttpOnly; SameSite=Strict; Path=/; Max-Age=43200{secure}",
            result.1
        )
        .parse()
        .unwrap(),
    );
    Ok(response)
}

#[cfg(test)]
mod proxy_tests {
    use super::*;
    #[test]
    fn direct_clients_cannot_spoof_their_rate_limit_address() {
        let mut h=HeaderMap::new(); h.insert("x-forwarded-for","198.51.100.9".parse().unwrap());
        assert_eq!(login_client(&h,"203.0.113.8:4000".parse().unwrap(),true),"203.0.113.8");
        assert_eq!(login_client(&h,"127.0.0.1:4000".parse().unwrap(),false),"127.0.0.1");
    }
    #[test]
    fn configured_local_proxy_uses_the_last_valid_address() {
        let mut h=HeaderMap::new(); h.insert("x-forwarded-for","untrusted, 198.51.100.9".parse().unwrap());
        assert_eq!(login_client(&h,"127.0.0.1:4000".parse().unwrap(),true),"198.51.100.9");
        h.insert("x-forwarded-for","invalid".parse().unwrap());
        assert_eq!(login_client(&h,"127.0.0.1:4000".parse().unwrap(),true),"127.0.0.1");
    }
}
async fn logout(h: HeaderMap) -> ApiResult {
    auth(&h, true)?;
    store()?.logout(&cookie(&h))?;
    let mut r = Json(json!({"ok":true})).into_response();
    r.headers_mut().insert(
        "set-cookie",
        "trail_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0"
            .parse()
            .unwrap(),
    );
    Ok(r)
}
async fn workspace(h: HeaderMap) -> ApiResult {
    let s = auth(&h, false)?;
    ok(store()?.workspace(&s.user)?)
}
async fn create(h: HeaderMap, Json(data): Json<Value>) -> ApiResult {
    let s = auth(&h, true)?;
    ok(store()?.create(&s.user, data["kind"].as_str().unwrap_or(""))?)
}
async fn change(h: HeaderMap, Path(id): Path<u64>, Json(data): Json<Change>) -> ApiResult {
    let s = auth(&h, true)?;
    ok(store()?.change(&s.user, id, data)?)
}
async fn preview(h: HeaderMap, Path(id): Path<u64>) -> ApiResult {
    let s = auth(&h, true)?;
    ok(json!({"url":store()?.preview(&s.user,id)?}))
}
async fn public() -> ApiResult {
    ok(store()?.public(None)?)
}
async fn download(Path(id): Path<u64>) -> ApiResult {
    let c = store()?.public(None)?;
    let item = c
        .items
        .into_iter()
        .find(|i| i.id == id && i.kind == "resource")
        .ok_or(ApiError("Not found.".into()))?;
    let url = item.field("url");
    if !db::valid_url(&url) || url.is_empty() {
        return Err(ApiError("Not found.".into()));
    }
    Ok(Redirect::temporary(&url).into_response())
}
async fn connections(h: HeaderMap, Json(data): Json<HashMap<String, String>>) -> ApiResult {
    let s = auth(&h, true)?;
    store()?.connections(&s.user, data)?;
    ok(json!({"ok":true}))
}
#[derive(Deserialize)]
struct NewUser {
    login: String,
    name: String,
    role: String,
    password: String,
}
async fn add_user(h: HeaderMap, Json(data): Json<NewUser>) -> ApiResult {
    let s = auth(&h, true)?;
    let db = store()?.clone();
    tokio::task::spawn_blocking(move || {
        db.add_user(&s.user, &data.login, &data.name, &data.role, &data.password)
    })
    .await
    .map_err(|_| ApiError("Could not create the account.".into()))??;
    ok(json!({"ok":true}))
}
async fn update_user(h: HeaderMap, Path(id): Path<u64>, Json(data): Json<Value>) -> ApiResult {
    let s = auth(&h, true)?;
    let db = store()?.clone();
    tokio::task::spawn_blocking(move || {
        if data["action"] == "remove" {
            db.remove_user(&s.user, id)
        } else {
            db.password(&s.user, id, data["password"].as_str().unwrap_or(""))
        }
    })
    .await
    .map_err(|_| ApiError("Could not update the account.".into()))??;
    ok(json!({"ok":true}))
}
async fn export(h: HeaderMap) -> ApiResult {
    let s = auth(&h, false)?;
    if s.user.role != "admin" {
        return Err(ApiError(
            "Only administrators can export all content.".into(),
        ));
    }
    let mut r = Json(store()?.workspace(&s.user)?).into_response();
    r.headers_mut().insert(
        "content-disposition",
        "attachment; filename=trail-content.json".parse().unwrap(),
    );
    Ok(r)
}
async fn upload(h: HeaderMap, mut form: Multipart) -> ApiResult {
    auth(&h, true)?;
    let field = form
        .next_field()
        .await
        .map_err(|_| ApiError("The upload could not be read.".into()))?
        .ok_or(ApiError("Select a file.".into()))?;
    let name = field
        .file_name()
        .unwrap_or("upload")
        .chars()
        .filter(|c| !c.is_control())
        .take(180)
        .collect::<String>();
    let bytes = field
        .bytes()
        .await
        .map_err(|_| ApiError("The file is too large or unreadable.".into()))?;
    if bytes.len() > 20 * 1024 * 1024 {
        return Err(ApiError("Choose a file smaller than 20 MB.".into()));
    }
    let (extension, mime) = if bytes.starts_with(b"%PDF-") {
        ("pdf", "application/pdf")
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        ("png", "image/png")
    } else if bytes.starts_with(&[255, 216, 255]) {
        ("jpg", "image/jpeg")
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        ("webp", "image/webp")
    } else {
        return Err(ApiError("Upload a PDF, PNG, JPEG or WebP file.".into()));
    };
    let id = format!("{}.{}", db::token(), extension);
    let m = Media {
        id: id.clone(),
        name,
        url: format!("/media/{id}"),
        mime: mime.into(),
        size: bytes.len() as u64,
    };
    std::fs::write(store()?.dir.join("uploads").join(&id), bytes)
        .map_err(|_| ApiError("Could not save the upload.".into()))?;
    store()?.add_media(&m)?;
    ok(m)
}
async fn media_file(Path(path): Path<String>) -> ApiResult {
    if path.split('/').any(|p| {
        p.is_empty()
            || p == "."
            || p == ".."
            || !p
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
    }) {
        return Err(ApiError("Not found.".into()));
    }
    let (root, file) = if let Some(p) = path.strip_prefix("legacy/") {
        ("legacy-uploads", p)
    } else {
        ("uploads", path.as_str())
    };
    let ext = file.rsplit('.').next().unwrap_or("").to_lowercase();
    let mime = match ext.as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        _ => return Err(ApiError("Not found.".into())),
    };
    let bytes = std::fs::read(store()?.dir.join(root).join(file))
        .map_err(|_| ApiError("Not found.".into()))?;
    let mut r = bytes.into_response();
    r.headers_mut()
        .insert("content-type", mime.parse().unwrap());
    if ext == "pdf" {
        r.headers_mut().insert(
            "content-disposition",
            "attachment; filename=trail-document.pdf".parse().unwrap(),
        );
    }
    Ok(r)
}
