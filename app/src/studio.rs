//! Trail Studio is a Leptos application, compiled to the same WASM bundle as the site.
use crate::studio_model::*;
#[path = "studio_photos.rs"]
mod photos;
#[path = "studio_library.rs"]
mod library;
#[path = "studio_support.rs"]
mod support;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::{use_location, use_navigate};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

async fn api<T: DeserializeOwned>(
    method: &str,
    path: &str,
    csrf: &str,
    body: Value,
) -> Result<T, String> {
    #[cfg(feature = "hydrate")]
    {
        let request = gloo_net::http::RequestBuilder::new(path)
            .method(method.parse().unwrap())
            .header("x-csrf-token", csrf);
        let response = if method == "GET" {
            request.send().await
        } else {
            request.json(&body).map_err(|e| e.to_string())?.send().await
        }
        .map_err(|_| {
            "Could not reach the server. Your draft remains in this browser.".to_string()
        })?;
        if !response.ok() {
            return Err(response
                .json::<Value>()
                .await
                .ok()
                .and_then(|v| v["error"].as_str().map(str::to_owned))
                .unwrap_or("The request could not be completed.".into()));
        }
        response.json().await.map_err(|e| e.to_string())
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (method, path, csrf, body);
        Err("Opening Trail Studio…".into())
    }
}
#[component]
pub fn Studio() -> impl IntoView {
    let session = RwSignal::new(None::<Session>);
    let checked = RwSignal::new(false);
    Effect::new(move |_| {
        spawn_local(async move {
            session.set(api("GET", "/api/cms/session", "", Value::Null).await.ok());
            checked.set(true);
        })
    });
    view! {<leptos_meta::Stylesheet id="studio-layout" href="/studio.css"/><leptos_meta::Stylesheet id="studio-native" href="/studio-native.css"/><leptos_meta::Stylesheet id="studio-editorial" href="/design/studio.css"/><leptos_meta::Title text="Trail Studio | Iditarod Historic Trail Alliance"/><div id="trail-studio"><Show when=move||checked.get() fallback=||view!{<div class="studio-loading"><h1>"Trail Studio"</h1><p>"Opening your workspace…"</p><noscript>"Enable JavaScript to edit content. The public website remains readable without it."</noscript></div>}>
    {move||match session.get(){Some(s)=>view!{<Dashboard session=s on_logout=Callback::new(move|_|session.set(None))/>}.into_any(),None=>view!{<Login on_login=Callback::new(move|s|session.set(Some(s)))/>}.into_any()}}
    </Show></div>}
}
#[component]
fn Login(on_login: Callback<Session>) -> impl IntoView {
    let login = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    view! {<main class="studio-login"><div class="login-photo"><span>"THE LIVING TRAIL"</span><h1>"A place to care for"<br/><em>"the trail’s story."</em></h1></div><form class="login-card" on:submit=move|ev|{ev.prevent_default();busy.set(true);spawn_local(async move{match api::<Session>("POST","/api/cms/login","",json!({"login":login.get_untracked(),"password":password.get_untracked()})).await{Ok(s)=>{password.set(String::new());on_login.run(s);},Err(e)=>error.set(e)}busy.set(false);});}><p class="eyebrow">"IDITAROD HISTORIC TRAIL ALLIANCE"</p><h2>"Welcome to Trail Studio"</h2><p>"Sign in to update the website."</p><div class="field"><label for="login">"Username"</label><input id="login" autocomplete="username" required on:input=move|e|login.set(event_target_value(&e))/></div><div class="field"><label for="password">"Password"</label><input id="password" type="password" autocomplete="current-password" required on:input=move|e|password.set(event_target_value(&e))/></div><p role="alert" class="form-error">{error}</p><button class="button primary full" disabled=busy>{move||if busy.get(){"Signing in…"}else{"Sign in"}}</button><a class="back-link" href="/">"← Back to the website"</a></form></main>}
}
#[derive(Clone, Copy)]
struct Ctx {
    session: StoredValue<Session>,
    refresh: RwSignal<u64>,
    notice: RwSignal<String>,
}
#[component]
fn Dashboard(session: Session, on_logout: Callback<()>) -> impl IntoView {
    let ctx = Ctx {
        session: StoredValue::new(session),
        refresh: RwSignal::new(0),
        notice: RwSignal::new(String::new()),
    };
    provide_context(ctx);
    let location = use_location();
    let path = location.pathname;
    let mobile = RwSignal::new(false);
    let workspace = LocalResource::new(move || {
        let _ = ctx.refresh.get();
        let _ = path.get();
        async move { api::<Workspace>("GET", "/api/cms/workspace", "", Value::Null).await }
    });
    let navigation=KINDS.iter().map(|(kind,label)|{let href=format!("/admin/{kind}");let active=href.clone();view!{<a href=href class:selected=move||path.get()==active on:click=move|_|mobile.set(false)><span class="nav-symbol">{match *kind {"page"=>"▤","story"=>"✎","place"=>"⌖","resource"=>"▱","event"=>"▦",_=>"♡"}}</span>{*label}</a>}}).collect_view();
    view! {<a class="studio-skip" href="#studio-main">"Skip to workspace"</a><aside class="studio-sidebar" class:open=mobile aria-label="Workspace navigation"><a class="studio-brand" href="/admin"><span class="brand-mark">"△"</span><span>"THE LIVING TRAIL"<small>"CONTENT STUDIO"</small></span></a><p class="workspace-label">"YOUR WORKSPACE"</p><nav aria-label="Workspace"><a href="/admin" class:selected=move||path.get()=="/admin" on:click=move|_|mobile.set(false)><span class="nav-symbol">"⌂"</span>"Overview"</a>{navigation}<div class="nav-rule"></div>{[ ("support","Support requests"),("media","Photos & files"),("connections","Connections"),("team","People & access"),("help","Help & guide")].into_iter().map(move|(slug,label)|{let href=format!("/admin/{slug}");let active=href.clone();view!{<a href=href class:selected=move||path.get()==active on:click=move|_|mobile.set(false)>{label}</a>}}).collect_view()}</nav><div class="sidebar-foot"><p>"Small updates."<br/>"A lasting connection."</p><a href="/" target="_blank" rel="noopener">"View website ↗"<span>"See what visitors see"</span></a></div></aside><div class="studio-workspace"><header class="studio-topbar"><button class="icon-button mobile-nav" aria-label="Toggle workspace menu" aria-expanded=move||mobile.get() on:click=move|_|mobile.update(|v|*v=!*v)>"☰"</button><div class="breadcrumbs">"Trail Studio"<span>"/"</span>"Your workspace"</div><div class="topbar-actions"><span class="staff-name">{ctx.session.get_value().user.name}</span><button class="button small" on:click=move|_|spawn_local(async move{match api::<Value>("POST","/api/cms/logout",&ctx.session.get_value().csrf,json!({})).await{Ok(_)=>on_logout.run(()),Err(e)=>ctx.notice.set(e)}})>"Sign out"</button></div></header><main id="studio-main" tabindex="-1"><Show when=move||!ctx.notice.get().is_empty()><p class="notice" role="status">{ctx.notice}</p></Show>{move||workspace.get().map(|r|match r{Err(e)=>view!{<p role="alert" class="notice error">{e}</p>}.into_any(),Ok(w)=>{let p=path.get();if let Some(id)=p.strip_prefix("/admin/edit/").and_then(|x|x.parse::<u64>().ok()){match w.items.iter().find(|i|i.content.id==id).cloned(){Some(item)=>view!{<Editor item media=w.media.clone() items=w.items.clone()/>}.into_any(),None=>view!{<h1>"Content not found"</h1>}.into_any()}}else{match p.trim_start_matches("/admin").trim_matches('/') {""=>view!{<Overview workspace=w/>}.into_any(),"support"=>view!{<support::SupportInbox/>}.into_any(),"media"=>view!{<MediaLibrary media=w.media/>}.into_any(),"connections"=>view!{<Connections values=w.connections/>}.into_any(),"team"=>view!{<Team users=w.users/>}.into_any(),"help"=>view!{<Help/>}.into_any(),kind=>view!{<ContentList kind=kind.to_owned() items=w.items/>}.into_any()}}}})}</main><footer class="studio-footer">"Made for the people who keep the trail’s story alive."</footer></div>}
}
fn ctx() -> Ctx {
    expect_context()
}
fn create_item(kind: String) {
    let c = ctx();
    let nav = use_navigate();
    spawn_local(async move {
        match api::<Item>(
            "POST",
            "/api/cms/items",
            &c.session.get_value().csrf,
            json!({"kind":kind}),
        )
        .await
        {
            Ok(i) => nav(&format!("/admin/edit/{}", i.content.id), Default::default()),
            Err(e) => c.notice.set(e),
        }
    });
}
#[component]
fn Overview(workspace: Workspace) -> impl IntoView {
    let published = workspace
        .items
        .iter()
        .filter(|i| i.status == "publish")
        .count();
    let drafts = workspace
        .items
        .iter()
        .filter(|i| i.draft.is_some() || i.status == "draft")
        .count();
    let scheduled = workspace
        .items
        .iter()
        .filter(|i| i.schedule.is_some())
        .count();
    let home = workspace
        .items
        .iter()
        .find(|i| i.content.kind == "page" && i.content.slug == "home")
        .map(|i| i.content.id);
    view! {<div class="page-heading"><div><p class="eyebrow">"YOUR WEBSITE, AT A GLANCE"</p><h1>"Welcome back."</h1><p>"A little care goes a long way. What would you like to share today?"</p></div><a class="button" href="/" target="_blank" rel="noopener">"View website ↗"</a></div><section class="welcome-card"><div class="welcome-copy"><p class="eyebrow">"KEEP THE CONNECTION GROWING"</p><h2>"Every story brings"<br/><em>"the trail a little closer."</em></h2><p>"Share a place, preserve a memory, or help someone find their way."</p><button class="button light" on:click=move|_|create_item("story".into())>"+ Write a story"</button></div><div class="welcome-image"><img src="/images/editorial/seward-harbor.webp" alt="Harbor and mountains in Seward, Alaska"/></div></section><div class="stats">{[(published,"Published items"),(drafts,"Drafts to work on"),(scheduled,"Scheduled updates"),(workspace.media.len(),"Uploaded files")].into_iter().map(|(n,label)|view!{<div class="stat"><strong>{n}</strong><span>{label}</span></div>}).collect_view()}</div><section class="quick-actions"><a href=home.map(|id|format!("/admin/edit/{id}")).unwrap_or("/admin/page".into())><span>"⌂"</span><strong>"Update the homepage"</strong><small>"Give visitors a warm welcome"</small></a><button on:click=move|_|create_item("resource".into())><span>"▱"</span><strong>"Add a document"</strong><small>"Keep useful information close"</small></button><button on:click=move|_|create_item("event".into())><span>"▦"</span><strong>"Share an event"</strong><small>"Bring the community together"</small></button></section><section class="panel"><div class="panel-heading"><h2>"Ready for your attention"</h2><a class="text-button" href="/admin/story">"All stories →"</a></div><ContentRows items=workspace.items.into_iter().filter(|i|i.draft.is_some()||i.status=="draft"||i.status=="pending").take(6).collect()/></section><div class="notice">"Drafts stay private until an editor publishes them. Preview changes before they go live."</div>}
}
#[component]
fn ContentList(kind: String, items: Vec<Item>) -> impl IntoView {
    let query = RwSignal::new(String::new());
    let filter = RwSignal::new("all".to_owned());
    let label = KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
        .unwrap_or("Content");
    let data = StoredValue::new(
        items
            .into_iter()
            .filter(|i| i.content.kind == kind)
            .collect::<Vec<_>>(),
    );
    view! {<div class="page-heading"><div><p class="eyebrow">"TELL THE TRAIL’S STORY"</p><h1>{label}</h1><p>"Find an update, pick up a draft, or start something new."</p></div><button class="button primary" on:click=move|_|create_item(kind.clone())>"+ Add new"</button></div><div class="list-controls"><label class="field"><span>"Search content"</span><input type="search" placeholder="Search by title…" on:input=move|e|query.set(event_target_value(&e))/></label><label class="field"><span>"Status"</span><select on:change=move|e|filter.set(event_target_value(&e))><option value="all">"All content"</option><option value="publish">"Published"</option><option value="draft">"Drafts"</option><option value="pending">"Needs review"</option><option value="private">"Archived"</option></select></label></div><section class="panel">{move||view!{<ContentRows items=data.get_value().into_iter().filter(|i|(filter.get()=="all"||i.status==filter.get()||(filter.get()=="draft"&&i.draft.is_some()))&&i.editable().title.to_lowercase().contains(&query.get().to_lowercase())).collect()/>}}</section>}
}
#[component]
fn ContentRows(items: Vec<Item>) -> impl IntoView {
    if items.is_empty() {
        return view!{<div class="empty-state"><h3>"A clear desk."</h3><p>"There’s no content in this view yet."</p></div>}.into_any();
    }
    view!{<div class="content-table"><table><thead><tr><th>"Title"</th><th>"Status"</th><th>"Updated"</th><th><span class="sr-only">"Edit"</span></th></tr></thead><tbody>{items.into_iter().map(|i|{let title=i.editable().title;view!{<tr><td><a class="content-title" href=format!("/admin/edit/{}",i.content.id)>{if title.is_empty(){"Untitled draft".into()}else{title}}</a><small class="row-kind">{i.content.kind}</small></td><td><span class="badge">{if i.schedule.is_some(){"Scheduled"}else if i.status=="publish"&&i.draft.is_some(){"Published · draft changes"}else{match i.status.as_str(){"publish"=>"Published","pending"=>"Needs review","private"=>"Archived",_=>"Draft"}}}</span></td><td>{if i.modified.starts_with("0000")||i.modified.is_empty(){"Not yet saved".to_owned()}else{i.modified.get(..10).unwrap_or(&i.modified).to_owned()}}</td><td><a class="text-button" href=format!("/admin/edit/{}",i.content.id)>"Edit →"</a></td></tr>}}).collect_view()}</tbody></table></div>}.into_any()
}

#[derive(Clone)]
struct EditState {
    item: Item,
    draft: Draft,
    generation: u64,
    dirty: bool,
    busy: bool,
    message: String,
    preview: String,
    at: String,
    blocked: bool,
}
#[path = "studio_sections.rs"]
mod sections;
type Edit = ArcRwSignal<EditState>;
fn field_value(d: &Draft, k: &str) -> String {
    match k {
        "title" => d.title.clone(),
        "summary" => d.summary.clone(),
        "body" => d.body.clone(),
        "slug" => d.slug.clone(),
        _ => d.field(k),
    }
}
fn update_field(s: &Edit, k: &str, value: String, user: u64) {
    s.update(|e| {
        match k {
            "title" => e.draft.title = value,
            "summary" => e.draft.summary = value,
            "body" => e.draft.body = value,
            "slug" => e.draft.slug = value,
            _ => {
                e.draft.fields.insert(k.into(), value);
            }
        }
        e.dirty = true;
        e.generation += 1;
        e.message = "Unsaved changes…".into();
    });
    store_recovery(s, user);
}
fn store_recovery(s: &Edit, user: u64) {
    #[cfg(feature = "hydrate")]
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let e = s.get_untracked();
        let _ = storage.set_item(
            &format!("trail-draft-{user}-{}", e.item.content.id),
            &json!({"version":e.item.version,"draft":e.draft}).to_string(),
        );
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (s, user);
}
async fn save_edit(
    s: Edit,
    session: Session,
    action: &str,
    revision: Option<String>,
) -> Result<(), String> {
    let e = s.get_untracked();
    if e.busy {
        return Err("Finishing the current save. Try again in a moment.".into());
    }
    if e.blocked {
        return Err("A newer version exists. Copy your changes, then reload this page.".into());
    }
    if action == "save" && !e.dirty {
        return Ok(());
    }
    s.update(|e| e.busy = true);
    let at = if e.at.is_empty() {
        None
    } else {
        Some(e.at.clone())
    };
    let c = Change {
        action: action.into(),
        version: e.item.version,
        content: Some(e.draft.clone()),
        at,
        revision,
    };
    let result = api::<Item>(
        "POST",
        &format!("/api/cms/items/{}", e.item.content.id),
        &session.csrf,
        serde_json::to_value(c).unwrap(),
    )
    .await;
    match result {
        Ok(item) => {
            s.update(|st| {
                st.item = item.clone();
                st.busy = false;
                if st.generation == e.generation {
                    st.dirty = false;
                    st.draft = item.editable();
                }
                st.message = match action {
                    "publish" => "Published. Your website is up to date.",
                    "schedule" => "Scheduled. The saved snapshot will publish at that time.",
                    "restore" => "Revision restored as a private draft.",
                    "archive" => "Archived. This content is no longer public.",
                    "submit" => "Sent to an editor for review.",
                    _ => "All changes saved.",
                }
                .into();
            });
            #[cfg(feature = "hydrate")]
            if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                if !s.get_untracked().dirty {
                    let _ = storage.remove_item(&format!(
                        "trail-draft-{}-{}",
                        session.user.id, e.item.content.id
                    ));
                } else {
                    store_recovery(&s, session.user.id);
                }
            }
            Ok(())
        }
        Err(error) => {
            s.update(|e| {
                e.busy = false;
                e.blocked = error.starts_with("Conflict:");
                e.message = error.clone();
            });
            Err(error)
        }
    }
}
fn perform(s: Edit, session: Session, action: &'static str, revision: Option<String>) {
    spawn_local(async move {
        if let Err(error) = save_edit(s.clone(), session, action, revision).await {
            s.update(|e| e.message = error);
        }
    });
}
#[component]
fn EditField(
    state: Edit,
    key: &'static str,
    label: &'static str,
    #[prop(default = false)] multiline: bool,
    #[prop(default = false)] readonly: bool,
    #[prop(default = "")] hint: &'static str,
) -> impl IntoView {
    let user = ctx().session.get_value().user.id;
    let read = state.clone();
    let write = state.clone();
    let id = format!("field-{key}");
    let description = format!("hint-{key}");
    view! {<div class="field"><label for=id.clone()>{label}</label>{if multiline{view!{<textarea id=id rows=if key=="body"{12}else{3} aria-describedby=description.clone() prop:value=move||field_value(&read.get().draft,key) on:input=move|e|update_field(&write,key,event_target_value(&e),user)></textarea>}.into_any()}else{view!{<input id=id readonly=readonly aria-describedby=description.clone() prop:value=move||field_value(&read.get().draft,key) on:input=move|e|update_field(&write,key,event_target_value(&e),user)/>}.into_any()}}<small id=description>{hint}</small></div>}
}
#[component]
fn ExplorerFields(state: Edit) -> impl IntoView {
    let user=ctx().session.get_value().user.id;
    let read=state.clone();let write=state.clone();
    view!{<section class="editor-panel"><h2>"Trail explorer"</h2><p class="small-note">"Connect this place to a verified point on the map. When published, its introduction, photo and the details below appear in the explorer. The geographic location stays fixed."</p><div class="field"><label for="field-map-stop">"Place on the map"</label><select id="field-map-stop" prop:value=move||field_value(&read.get().draft,"map_stop") on:change=move|e|update_field(&write,"map_stop",event_target_value(&e),user)><option value="">"Use the original field guide"</option>{crate::explorer::STOPS.iter().map(|(id,name)|view!{<option value=*id>{*name}</option>}).collect_view()}</select></div><EditField state=state.clone() key="trail_history" label="Explorer history" multiline=true hint="Leave blank to keep the researched history."/><EditField state=state.clone() key="history_url" label="History source address" hint="Link to the source for your history."/><EditField state=state.clone() key="summer_activities" label="Summer ideas" multiline=true hint="One activity per line, up to six. Local planning guidance remains visible."/><EditField state=state.clone() key="winter_activities" label="Winter ideas" multiline=true hint="One activity per line, up to six."/><EditField state=state key="url" label="Visitor information address"/></section>}
}
#[component]
fn Editor(item: Item, media: Vec<Media>, items: Vec<Item>) -> impl IntoView {
    let session = ctx().session.get_value();
    let s = ArcRwSignal::new(EditState {
        draft: item.editable(),
        item: item.clone(),
        generation: 0,
        dirty: false,
        busy: false,
        message: "All changes saved.".into(),
        preview: String::new(),
        at: String::new(),
        blocked: false,
    });
    let session = StoredValue::new(session);
    let state = StoredValue::new(s.clone());
    let kind = item.content.kind.clone();
    let title = KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map(|(_, v)| *v)
        .unwrap_or("Content");
    #[cfg(feature = "hydrate")]
    {
        if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            if let Ok(Some(raw)) = storage.get_item(&format!(
                "trail-draft-{}-{}",
                session.get_value().user.id,
                item.content.id
            )) {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    if let Ok(draft) = serde_json::from_value::<Draft>(v["draft"].clone()) {
                        s.update(|e|{e.draft=draft;e.dirty=true;e.generation=1;e.blocked=v["version"].as_u64()!=Some(item.version);e.message=if e.blocked{"Recovered browser changes. A newer server version exists; copy your changes before reloading."}else{"Recovered your unsaved browser draft."}.into();});
                    }
                }
            }
        }
    }
    let autosave = s.clone();
    let autosession = session.get_value();
    Effect::new(move |_| {
        let e = autosave.get();
        if !e.dirty || e.busy || e.blocked {
            return;
        }
        let s = autosave.clone();
        let session = autosession.clone();
        spawn_local(async move {
            #[cfg(feature = "hydrate")]
            gloo_timers::future::TimeoutFuture::new(850).await;
            if s.get_untracked().generation == e.generation {
                let _ = save_edit(s, session, "save", None).await;
            }
        });
    });
    let read = s.clone();
    let dirty = s.clone();
    let busy = s.clone();
    let preview = s.clone();
    let schedule = s.clone();
    let revisions = s.clone();
    view! {<div class="editor-top"><div><a class="text-button" href=format!("/admin/{kind}")>{format!("← {title}")}</a><h1>{move||{let title=state.get_value().get().draft.title;if title.is_empty(){"A new beginning".to_owned()}else{title}}}</h1><p id="save-status" role="status">{move||read.get().message}</p></div><div class="editor-actions"><button class="button" disabled=move||dirty.get().busy on:click=move|_|perform(state.get_value(),session.get_value(),"save",None)>"Save draft"</button><button class="button" disabled=move||busy.get().busy on:click=move|_|{let s=state.get_value();let session=session.get_value();spawn_local(async move{if save_edit(s.clone(),session.clone(),"save",None).await.is_ok(){match api::<Value>("POST",&format!("/api/cms/preview/{}",s.get_untracked().item.content.id),&session.csrf,json!({})).await{Ok(v)=>s.update(|e|e.preview=v["url"].as_str().unwrap_or("").into()),Err(error)=>s.update(|e|e.message=error)}}});}>"Preview"</button></div></div><nav class="editor-section-nav" aria-label="Editor sections"><a href="#editor-writing">"Words"</a><a href="#editor-photo">"Photography"</a><a href="#page-sections">"Page sections"</a><a href="#editor-publishing">"Publishing"</a></nav><div class="editor-layout"><div><section class="editor-panel" id="editor-writing"><h2>"The story you want to tell"</h2><p class="small-note">"Clear words and a good photo are a great place to start."</p><EditField state=s.clone() key="title" label="Headline"/><EditField state=s.clone() key="summary" label="Short introduction" multiline=true/><EditField state=s.clone() key="category" label="Category or section label"/><EditField state=s.clone() key="body" label="Body text" multiline=true hint="Write in paragraphs. Text is displayed safely, without HTML code."/></section><photos::PhotoControls state=s.clone() media=media.clone() allow_layout={kind=="place"||kind=="story"||item.content.slug=="about"}/>
    {if kind=="resource"{view!{<section class="editor-panel"><h2>"Document"</h2><EditField state=s.clone() key="url" label="Document or resource address" hint="Upload a PDF in Photos & files, then paste its address here. Replacing it keeps the public download link unchanged."/><EditField state=s.clone() key="format" label="File format"/><div class="stable-link">"Permanent download link: "<code>{format!("/api/cms/download/{}",item.content.id)}</code></div></section>}.into_any()}else if kind=="place"{view!{<section class="editor-panel"><h2>"Finding this place"</h2><EditField state=s.clone() key="region" label="Region"/><EditField state=s.clone() key="season" label="Season" hint="Use Winter, Summer or All seasons."/><EditField state=s.clone() key="activity" label="Activities"/><EditField state=s.clone() key="visit_notes" label="Visit introduction" multiline=true hint="Optional destination-guide introduction. Blank keeps the researched guide."/><EditField state=s.clone() key="season_notes" label="Seasonal planning notes" multiline=true/><EditField state=s.clone() key="access_notes" label="Access and preparation" multiline=true/></section><ExplorerFields state=s.clone()/>}.into_any()}else if kind=="event"{view!{<section class="editor-panel"><h2>"Event details"</h2><EditField state=s.clone() key="start" label="Start date" hint="YYYY-MM-DD or a full timestamp with time zone."/><EditField state=s.clone() key="end" label="End date" hint="The event leaves listings after this date."/><EditField state=s.clone() key="location" label="Location"/></section>}.into_any()}else{().into_any()}}
    {if kind=="page"&&item.content.slug=="home"{view!{<section class="editor-panel"><h2>"Homepage highlights"</h2><EditField state=s.clone() key="headline_accent" label="Second headline line"/><photos::SummerPhoto state=s.clone()/><FeaturedChoices state=s.clone() items=items.clone()/></section>}.into_any()}else{().into_any()}}
    {(kind=="story").then(||view!{<library::EditorialPhotos state=s.clone()/>})}
    {(kind=="campaign").then(||view!{<section class="editor-panel"><h2>"Project record"</h2><p class="small-note">"Published entries with a project status appear under Work supported. Use documented outcomes and dates. Describe future help as an invitation, not an unconfirmed vacancy."</p><EditField state=s.clone() key="project_status" label="Project status" hint="For example: Completed, or Repair funding documented. Leave blank for a general giving page."/><EditField state=s.clone() key="project_date" label="Project date or reporting period"/><EditField state=s.clone() key="history_url" label="Project source address"/><EditField state=s.clone() key="supported_costs" label="What support covered" multiline=true/><EditField state=s.clone() key="help_needed" label="How people could help next" multiline=true/><EditField state=s.clone() key="support_interest" label="Support interest key" hint="Use trail-care, history, education, or where-needed."/></section>})}<sections::PageSectionEditor state=s.clone() media=media.clone()/><section class="editor-panel"><h2>"Optional button"</h2><EditField state=s.clone() key="button_text" label="Button text"/><EditField state=s.clone() key="button_url" label="Button destination"/></section></div><aside class="editor-aside" id="editor-publishing" aria-label="Publishing settings"><sections::PublishingChecks state=s.clone()/><section class="editor-panel"><h2>"Ready to share?"</h2><p class="small-note">"Preview your work, then publish when it feels right. Saving a draft never changes the live website."</p><button class="button primary full" disabled=move||state.get_value().get().busy on:click=move|_|perform(state.get_value(),session.get_value(),if session.get_value().user.publisher(){"publish"}else{"submit"},None)>{if session.get_value().user.publisher(){"Publish changes"}else{"Send for review"}}</button>
    {if session.get_value().user.publisher(){view!{<div class="field"><label for="schedule-at">"Publish later"</label><input id="schedule-at" type="datetime-local" on:change=move|e|{let value=event_target_value(&e);#[cfg(feature="hydrate")]let value=js_sys::Date::new(&value.into()).to_iso_string().as_string().unwrap_or_default();state.get_value().update(|e|e.at=value);}/><small>"Choose a time in your local time zone. Scheduling freezes a copy of this draft."</small></div><button class="button full" on:click=move|_|perform(state.get_value(),session.get_value(),"schedule",None)>"Schedule publication"</button>{move||schedule.get().item.schedule.map(|s|view!{<div class="schedule-note"><p>{format!("Scheduled: {}",s.at)}</p><button class="text-button" on:click=move|_|perform(state.get_value(),session.get_value(),"cancel-schedule",None)>"Cancel schedule"</button></div>})}}.into_any()}else{().into_any()}}
    </section><section class="editor-panel"><h2>"Address & search"</h2><EditField state=s.clone() key="slug" label="Address ending" readonly=item.ever_published hint="Use lowercase words separated by hyphens. Published addresses stay permanent."/><EditField state=s.clone() key="seo_title" label="Search result title"/><EditField state=s.clone() key="seo_description" label="Search result description" multiline=true/></section><section class="editor-panel"><h2>"Earlier versions"</h2><p class="small-note">"Restoring creates a private draft. Publish it when ready."</p>{move||revisions.get().item.history.iter().take(12).cloned().map(|r|view!{<div class="revision"><small>{format!("{} · {}",r.at.get(..16).unwrap_or(""),r.label)}</small><button class="text-button" on:click=move|_|perform(state.get_value(),session.get_value(),"restore",Some(r.id.clone()))>"Restore as draft"</button></div>}).collect_view()}<button class="button full" on:click=move|_|perform(state.get_value(),session.get_value(),"discard",None)>"Discard draft changes"</button></section>{if session.get_value().user.publisher(){view!{<section class="editor-panel"><h2>"Archive"</h2><p class="small-note">"Remove this item from the website. Its content and revisions are kept."</p><button class="button danger full" on:click=move|_|perform(state.get_value(),session.get_value(),"archive",None)>"Archive content"</button></section>}.into_any()}else{().into_any()}}</aside></div>
    <Show when=move||!preview.get().preview.is_empty()><PreviewDialog source=Signal::derive(move||state.get_value().get().preview) on_close=Callback::new(move|_|state.get_value().update(|e|e.preview.clear()))/></Show>}
}
#[component]
fn PreviewDialog(source: Signal<String>, on_close: Callback<()>) -> impl IntoView {
    view!{<sections::ResponsivePreview source on_close/>}
}

#[component]
fn FeaturedChoices(state: Edit, items: Vec<Item>) -> impl IntoView {
    let user = ctx().session.get_value().user.id;
    let read = state.clone();
    let write = state.clone();
    let stories = items
        .iter()
        .filter(|i| i.content.kind == "story" && i.status == "publish")
        .cloned()
        .collect::<Vec<_>>();
    let places = items
        .into_iter()
        .filter(|i| i.content.kind == "place" && i.status == "publish")
        .collect::<Vec<_>>();
    view! {<div class="field"><label for="featured-story">"Featured story"</label><select id="featured-story" prop:value=move||read.get().draft.field("featured_story") on:change=move|e|update_field(&write,"featured_story",event_target_value(&e),user)><option value="">"Choose a story"</option>{stories.into_iter().map(|i|view!{<option value=i.content.id.to_string()>{i.content.title}</option>}).collect_view()}</select></div><fieldset class="field"><legend>"Featured places"</legend><div class="place-choices">{places.into_iter().map(|i|{let read=state.clone();let write=state.clone();let id=i.content.id.to_string();let id2=id.clone();view!{<label><input type="checkbox" prop:checked=move||read.get().draft.field("featured_places").split(',').any(|p|p.trim()==id) on:change=move|e|{let mut ids=write.get_untracked().draft.field("featured_places").split(',').map(str::trim).filter(|p|!p.is_empty()).map(str::to_owned).collect::<Vec<_>>();ids.retain(|p|p!=&id2);if event_target_checked(&e){ids.push(id2.clone());}update_field(&write,"featured_places",ids.join(","),user);}/>{i.content.title}</label>}}).collect_view()}</div></fieldset>}
}

#[component]
fn MediaLibrary(media: Vec<Media>) -> impl IntoView {
    #[cfg(feature = "hydrate")]
    let c = ctx();
    let busy = RwSignal::new(false);
    view! {<div class="page-heading"><div><p class="eyebrow">"THE TRAIL, IN PICTURES & WORDS"</p><h1>"Photos & files"</h1><p>"Upload once, then use the address in your pages and stories."</p></div></div><section class="editor-panel"><h2>"Add a file"</h2><label class="field"><span>"Choose a photo or PDF"</span><input type="file" aria-label="Choose a photo or PDF" accept="image/png,image/jpeg,image/webp,application/pdf" disabled=busy on:change=move|ev|{
    #[cfg(feature="hydrate")]{use wasm_bindgen::JsCast;let input=ev.target().unwrap().unchecked_into::<web_sys::HtmlInputElement>();if let Some(file)=input.files().and_then(|f|f.get(0)){busy.set(true);let csrf=c.session.get_value().csrf;spawn_local(async move{let form=web_sys::FormData::new().unwrap();let _=form.append_with_blob_and_filename("file",&file,&file.name());let result=async {let response=gloo_net::http::Request::post("/api/cms/media").header("x-csrf-token",&csrf).body(form).map_err(|e|e.to_string())?.send().await.map_err(|e|e.to_string())?;if response.ok(){Ok(())}else{Err(response.json::<Value>().await.unwrap_or_default()["error"].as_str().unwrap_or("Upload failed.").to_owned())}}.await;match result{Ok(())=>{c.notice.set("File uploaded. Copy its address below.".into());c.refresh.update(|n|*n+=1);},Err(e)=>c.notice.set(e)}busy.set(false);});}}
    #[cfg(not(feature="hydrate"))]let _=ev;
    }/><small>"PDF, JPEG, PNG or WebP · Up to 20 MB. Add a photo description when using an image in content."</small></label><p role="status">{move||if busy.get(){"Uploading…"}else{""}}</p></section><div class="media-grid">{media.into_iter().map(|m|view!{<article class="media-card">{if m.mime.starts_with("image/"){view!{<img src=m.url.clone() alt=""/>}.into_any()}else{view!{<div class="document-icon">"PDF"</div>}.into_any()}}<h2>{m.name}</h2><p>{format!("{} KB",m.size.div_ceil(1024))}</p><label class="field"><span>"File address"</span><input readonly prop:value=m.url.clone() on:focus=move|e|{#[cfg(feature="hydrate")]{use wasm_bindgen::JsCast;if let Some(t)=e.target(){t.unchecked_into::<web_sys::HtmlInputElement>().select();}}#[cfg(not(feature="hydrate"))]let _=e;}/></label><a class="text-button" href=m.url target="_blank" rel="noopener">"Open file ↗"</a></article>}).collect_view()}</div>}
}
#[component]
fn Connections(values: std::collections::HashMap<String, String>) -> impl IntoView {
    let c = ctx();
    let values = RwSignal::new(values);
    let editable = c.session.get_value().user.role == "admin";
    let busy = RwSignal::new(false);
    view! {<div class="page-heading"><div><p class="eyebrow">"THE NEXT CONNECTION"</p><h1>"Connections"</h1><p>"Add approved destinations when your team is ready."</p></div></div>
    <form class="connections-form" on:submit=move|ev|{ev.prevent_default();busy.set(true);spawn_local(async move{let data:std::collections::HashMap<String,String>=values.get_untracked().into_iter().map(|(k,v)|(k,v.trim().to_owned())).collect();match api::<Value>("POST","/api/cms/connections",&c.session.get_value().csrf,serde_json::to_value(&data).unwrap()).await{Ok(_)=>{values.set(data);c.notice.set("Connections saved. Public website links are up to date.".into());},Err(e)=>c.notice.set(e)}busy.set(false);});}>
    <section class="editor-panel"><div class="connection-section-heading"><span class="eyebrow">"ON THE PUBLIC WEBSITE"</span><h2>"Visitor destinations"</h2><p class="small-note">"Paste the public HTTPS links for your store, checkout, or signup pages. Saving makes these links visible on the website. Leave a field blank to use the website’s enquiry or interest form; the shop stays closed until its destination is added."</p></div>
    {PUBLIC_LINKS.iter().map(|&(key,label)|view!{<ConnectionField values key label editable public=true/>}).collect_view()}
    </section><section class="editor-panel"><div class="connection-section-heading"><span class="eyebrow">"FOR YOUR TEAM"</span><h2>"Service shortcuts"</h2><p class="small-note">"These links stay inside Trail Studio. Payment records and active mailing lists remain with their providers. Website enquiries appear under Support requests. Store website addresses here, never passwords or API keys."</p></div>
    {[("square","Square dashboard"),("donors","Donor & member records"),("email","Email & newsletters")].into_iter().map(|(key,label)|view!{<ConnectionField values key label editable/>}).collect_view()}
    </section>{editable.then(||view!{<button class="button primary" type="submit" disabled=busy>{move||if busy.get(){"Saving connections..."}else{"Save connections"}}</button>})}</form>}

}
#[component]
fn ConnectionField(values: RwSignal<std::collections::HashMap<String,String>>, key: &'static str, label: &'static str, editable: bool, #[prop(default=false)] public: bool) -> impl IntoView {
    view!{<div class="connection-field field"><div><label for=key>{label}</label><span class="connection-state" class:configured=move||values.get().get(key).is_some_and(|v|!v.trim().is_empty())>{move||if values.get().get(key).is_some_and(|v|!v.trim().is_empty()){"Address added"}else if public{"Not activated"}else{"Not added"}}</span></div><input id=key type="url" readonly=!editable maxlength="2000" prop:value=move||values.get().get(key).cloned().unwrap_or_default() placeholder="https://" on:input=move|e|values.update(|v|{v.insert(key.into(),event_target_value(&e));})/><Show when=move||values.get().get(key).is_some_and(|v|v.starts_with("https://")&&v.len()>8)><a class="text-button" href=move||values.get().get(key).cloned().unwrap_or_default() target="_blank" rel="noopener noreferrer">"Check destination"</a></Show></div>}
}

#[component]
fn Team(users: Vec<User>) -> impl IntoView {
    let c = ctx();
    let admin = c.session.get_value().user.role == "admin";
    let login = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let role = RwSignal::new("contributor".to_owned());
    let password = RwSignal::new(String::new());
    let reset = RwSignal::new(String::new());
    let target = RwSignal::new(c.session.get_value().user.id);
    view! {<div class="page-heading"><div><p class="eyebrow">"CARE IS A TEAM EFFORT"</p><h1>"People & access"</h1><p>"Contributors prepare drafts. Editors publish. Administrators manage access."</p></div></div><section class="editor-panel"><h2>"Your team"</h2>{users.iter().cloned().map(|u|{let own=u.id==c.session.get_value().user.id;view!{<div class="team-row"><span><strong>{u.name}</strong><small>{format!("{} · {}",u.login,u.role)}</small></span>{(admin&&!own).then(||view!{<button class="button small danger" on:click=move|_|spawn_local(async move{match api::<Value>("POST",&format!("/api/cms/users/{}",u.id),&c.session.get_value().csrf,json!({"action":"remove"})).await{Ok(_)=>{c.notice.set("Access removed. Their content is preserved.".into());c.refresh.update(|n|*n+=1);},Err(e)=>c.notice.set(e)}})>"Remove access"</button>})}</div>}}).collect_view()}</section>{admin.then(||view!{<form class="editor-panel" on:submit=move|e|{e.prevent_default();spawn_local(async move{match api::<Value>("POST","/api/cms/users",&c.session.get_value().csrf,json!({"login":login.get_untracked(),"name":name.get_untracked(),"role":role.get_untracked(),"password":password.get_untracked()})).await{Ok(_)=>{password.set(String::new());c.notice.set("Account created. Share its password privately; no email was sent.".into());c.refresh.update(|n|*n+=1);},Err(e)=>c.notice.set(e)}});} ><h2>"Add a staff member"</h2><label class="field"><span>"Full name"</span><input required on:input=move|e|name.set(event_target_value(&e))/></label><label class="field"><span>"Username"</span><input required autocomplete="off" on:input=move|e|login.set(event_target_value(&e))/></label><label class="field"><span>"Role"</span><select on:change=move|e|role.set(event_target_value(&e))><option value="contributor">"Contributor — own drafts"</option><option value="editor">"Editor — all content & publishing"</option><option value="admin">"Administrator — content & accounts"</option></select></label><label class="field"><span>"Initial password"</span><input required type="password" minlength="12" autocomplete="new-password" prop:value=password on:input=move|e|password.set(event_target_value(&e))/><small>"At least 12 characters. Share it privately with this person."</small></label><button class="button primary">"Create account"</button></form>})}<form class="editor-panel" on:submit=move|e|{e.prevent_default();spawn_local(async move{match api::<Value>("POST",&format!("/api/cms/users/{}",target.get_untracked()),&c.session.get_value().csrf,json!({"action":"password","password":reset.get_untracked()})).await{Ok(_)=>{reset.set(String::new());c.notice.set("Password updated. That person's sessions were signed out. Sign in again if this is your account.".into());},Err(e)=>c.notice.set(e)}});} ><h2>"Change a password"</h2><label class="field"><span>"Account"</span><select prop:value=target.get_untracked().to_string() on:change=move|e|target.set(event_target_value(&e).parse().unwrap_or(0))>{users.into_iter().map(|u|view!{<option value=u.id.to_string()>{u.name}</option>}).collect_view()}</select></label><label class="field"><span>"New password"</span><input type="password" minlength="12" required autocomplete="new-password" prop:value=reset on:input=move|e|reset.set(event_target_value(&e))/></label><button class="button">"Update password"</button></form>}
}
#[component]
fn Help() -> impl IntoView {
    view! {<div class="page-heading"><div><p class="eyebrow">"A LITTLE GUIDANCE"</p><h1>"You’ve got this."</h1><p>"The everyday guide to caring for your website."</p></div></div><div class="help-grid">{[("1. Make a small change","Open a page or story, edit the fields, and wait for “All changes saved.” Your private draft is saved automatically. If a connection drops, this browser keeps a recovery copy."),("2. See it as visitors will","Choose Preview to see the actual website template. Preview links expire after 30 minutes; anyone with the link can view that snapshot."),("Compose a page","Choose an approved section: Editorial text, Captioned photograph, Photo & story, or Invitation. Move sections up or down, hide them, or undo a removal. Sections follow the standard page content, and all changes stay private until publication."),("Keep photographs useful","Choose from the credited collection or uploaded files. Add alt text, captions and credits, then use the focus sliders. Preview at Desktop, Tablet and Phone sizes to check the same components visitors will see."),("3. Publish when ready","Editors can publish immediately or choose a future time. A schedule uses a frozen copy, so later edits stay private. Contributors send drafts for review."),("Replace a document","Upload a PDF under Photos & files. Copy its address into the document’s resource address, then publish. The permanent download link stays the same."),("Recover an earlier version","The editor keeps earlier published versions. Restore one as a draft, check the preview, then publish it."),("Back up the workspace","Administrators can export content below. The server backup script also saves the SQLite database, staff accounts, settings and uploads. Keep a copy outside this computer.")].into_iter().map(|(title,text)|view!{<article class="editor-panel"><h2>{title}</h2><p>{text}</p></article>}).collect_view()}</div><a class="button" href="/api/cms/export" download="trail-content.json">"Export content JSON"</a>}
}
