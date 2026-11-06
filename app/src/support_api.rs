use crate::{studio_api::{auth,origin,login_client,ApiError}, studio_store::{store,token}, support_model::*};
use axum::{Router,Json,extract::{Path,ConnectInfo,DefaultBodyLimit},http::HeaderMap,routing::{get,post},response::{IntoResponse,Response}};
use serde_json::json;
use std::net::SocketAddr;
type Result=std::result::Result<Response,ApiError>;
pub fn routes()->Router<leptos::prelude::LeptosOptions> {
    Router::new().route("/api/support/start",get(||async{Json(json!({"key":token()}))}))
        .route("/api/support/requests",post(submit).layer(DefaultBodyLimit::max(16384)))
        .route("/api/support/receipt/{key}",get(receipt))
        .route("/api/support/withdraw/{key}",post(withdraw))
        .route("/api/cms/support",get(list))
        .route("/api/cms/support/export/{kind}",get(export))
        .route("/api/cms/support/{id}",post(change).layer(DefaultBodyLimit::max(8192)))
}
async fn submit(h:HeaderMap,ConnectInfo(peer):ConnectInfo<SocketAddr>,Json(input):Json<Submission>)->Result {
    origin(&h)?;let client=login_client(&h,peer,std::env::var("TRAIL_TRUST_PROXY").as_deref()==Ok("1"));Ok(Json(store()?.support_submit(input,&client)?).into_response())
}
async fn receipt(Path(key):Path<String>)->Result {Ok(Json(store()?.support_receipt(&key)?).into_response())}
async fn withdraw(h:HeaderMap,Path(key):Path<String>)->Result {origin(&h)?;Ok(Json(store()?.support_withdraw(&key)?).into_response())}
async fn list(h:HeaderMap)->Result {let s=auth(&h,false)?;Ok(Json(store()?.support_list(&s.user)?).into_response())}
async fn change(h:HeaderMap,Path(id):Path<u64>,Json(input):Json<SupportChange>)->Result {let s=auth(&h,true)?;store()?.support_change(&s.user,id,input)?;Ok(Json(json!({"ok":true})).into_response())}
async fn export(h:HeaderMap,Path(kind):Path<String>)->Result {let s=auth(&h,false)?;let csv=store()?.support_csv(&s.user,&kind)?;let mut r=csv.into_response();r.headers_mut().insert("content-type","text/csv; charset=utf-8".parse().unwrap());r.headers_mut().insert("content-disposition",format!("attachment; filename=trail-{kind}-requests.csv").parse().unwrap());Ok(r)}
