//! Native SQLite content is loaded on the server and serialized for Leptos hydration.
use crate::{components::*, pages, paths::url};
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::{use_location, use_params_map};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Content {
    #[serde(default)]
    pub id: u64,
    #[serde(rename = "type")]
    pub kind: String,
    pub slug: String,
    pub title: String,
    pub summary: String,
    pub body: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub fields: HashMap<String, String>,
}
impl Content {
    pub fn field(&self, key: &str) -> String {
        self.fields.get(key).cloned().unwrap_or_default()
    }
    pub fn image(&self) -> String {
        url(&self.field("image"))
    }
    pub fn image_position(&self) -> String {
        let x = self.field("image_x").parse::<u8>().unwrap_or(50).min(100);
        let y = self.field("image_y").parse::<u8>().unwrap_or(50).min(100);
        format!("object-position:{x}% {y}%")
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub items: Vec<Content>,
    #[serde(default)]
    pub preview_id: Option<u64>,
    #[serde(default)]
    pub download_base: String,
    #[serde(default)]
    pub public_links: HashMap<String, String>,
}
#[derive(Clone, Copy)]
pub struct CmsData(pub StoredValue<Option<Catalog>>);
pub fn catalog() -> Option<Catalog> {
    use_context::<CmsData>().and_then(|data| data.0.get_value())
}
pub fn public_link(key: &str) -> Option<String> {
    catalog()?.public_links.get(key).cloned().filter(|v| !v.is_empty())
}
pub fn page(slug: &str) -> Option<Content> {
    catalog()?
        .items
        .into_iter()
        .find(|i| i.kind == "page" && i.slug == slug)
}
pub fn items(kind: &str) -> Vec<Content> {
    catalog()
        .map(|c| c.items.into_iter().filter(|i| i.kind == kind).collect())
        .unwrap_or_default()
}

#[server]
pub async fn load_content(preview: Option<String>) -> Result<Option<Catalog>, ServerFnError> {
    let Some(store) = crate::studio_store::STORE.get() else {
        return Ok(None);
    };
    if let Some(token) = &preview {
        if token.len() != 48 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(ServerFnError::new("Invalid preview link."));
        }
    }
    store.public(preview).map(Some).map_err(ServerFnError::new)
}

#[component]
pub fn CmsGate() -> impl IntoView {
    let location = use_location();
    let token = Memo::new(move |_| {
        location
            .pathname
            .get()
            .split("/cms-preview/")
            .nth(1)
            .map(str::to_owned)
    });
    let content = Resource::new(move || token.get(), load_content);
    view! {<Suspense fallback=||view!{<p class="container section">"Opening the trail…"</p>}>
        {move||content.get().map(|result|match result {
            Ok(data)=>view!{<crate::app::SiteContent data/>}.into_any(),
            Err(error)=>view!{<section class="container section empty-state"><h1>"A short pause on the trail."</h1><p>{error.to_string()}</p><a class="button button-green" href=url("/")>"Return to the website"</a></section>}.into_any(),
        })}
    </Suspense>}
}

#[component]
pub fn ContentMeta(item: Content) -> impl IntoView {
    let title = if item.field("seo_title").is_empty() {
        item.title.clone()
    } else {
        item.field("seo_title")
    };
    let summary = if item.field("seo_description").is_empty() {
        item.summary.clone()
    } else {
        item.field("seo_description")
    };
    view! {<Title text=format!("{title} | The Living Trail")/><Meta name="description" content=summary/>}
}

#[component]
pub fn BodyText(text: String) -> impl IntoView {
    view! {<>{text.split("\n\n").filter(|p|!p.trim().is_empty()).map(|p|view!{<p>{p.to_string()}</p>}).collect_view()}</>}
}

#[component]
pub fn ContentCard(item: Content) -> impl IntoView {
    let href = url(&item.path);
    let image = crate::photography::thumbnail(&item);
    view! {<a class="story-card cms-card" href=href>
        {(!item.field("image").is_empty()).then(||view!{<div class="card-image"><img src=image srcset=crate::photography::srcset(&item) sizes="(max-width: 760px) 100vw, 40vw" style=item.image_position() alt=item.field("image_alt") loading="lazy" width="800" height="540"/><span class="circle-arrow"><Icon/></span></div>})}
        <p class="eyebrow">{item.field("category")}</p><h3>{item.title}</h3><p>{item.summary}</p><span class="text-link">"Explore more"<Icon/></span>
    </a>}
}

#[component]
pub fn ManagedHomePlaces() -> impl IntoView {
    let home = page("home").unwrap_or_default();
    let all = items("place");
    let chosen: Vec<u64> = home
        .field("featured_places")
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    let places = if chosen.is_empty() {
        all
    } else {
        chosen
            .iter()
            .filter_map(|id| all.iter().find(|p| p.id == *id).cloned())
            .collect()
    };
    view! {<div class="three-grid">{places.into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div>}
}
#[component]
pub fn ManagedFeature() -> impl IntoView {
    let home = page("home").unwrap_or_default();
    let id = home.field("featured_story").parse::<u64>().ok();
    let stories = items("story");
    let story = stories
        .iter()
        .find(|s| Some(s.id) == id)
        .or_else(|| stories.iter().find(|s| s.slug == "the-winter-way"))
        .or(stories.first())
        .cloned();
    view! {<>{story.map(|item|view!{<section class="feature-story"><img src=item.image() style=item.image_position() alt=item.field("image_alt") loading="lazy"/><div class="feature-story-shade"/><div class="container"><div><p class="eyebrow"><span class="small-line"/>{item.field("category")}</p><h2>{item.title.clone()}</h2><p>{item.summary.clone()}</p><a href=url(&item.path) class="button button-light">"Read the story"<Icon/></a></div></div></section>})}</>}
}

#[component]
pub fn ManagedIntro(slug: &'static str, #[prop(default=true)] show_photo: bool) -> impl IntoView {
    let item = page(slug).unwrap_or_default();
    view! {<ContentMeta item=item.clone()/><section class="page-intro container"><p class="eyebrow"><span class="small-line"/>{item.field("category")}</p><h1>{item.title.clone()}</h1><p class="intro-copy">{item.summary.clone()}</p></section>
        {(show_photo&&!item.field("image").is_empty()).then(||view!{<crate::interiors::HeroPhoto item=item.clone()/>})}
        {(!item.body.is_empty()).then(||view!{<section class="container prose cms-intro-body"><BodyText text=item.body.clone()/></section>})}
        {(!item.field("button_url").is_empty()).then(||view!{<div class="container cms-intro-action"><a class="button button-green" href=url(&item.field("button_url"))>{item.field("button_text")}<Icon/></a></div>})}
    }
}

#[component]
pub fn ManagedExplore() -> impl IntoView {
    if catalog().is_none() {
        return view! {<pages::Explore/>}.into_any();
    }
    let places = StoredValue::new(items("place"));
    let season = crate::explorer::trail_state().season;
    let region = RwSignal::new("All regions".to_string());
    let query = RwSignal::new(String::new());
    let mut regions: Vec<_> = places
        .get_value()
        .iter()
        .map(|i| i.field("region"))
        .filter(|s| !s.is_empty())
        .collect();
    regions.sort();
    regions.dedup();
    let filtered = Memo::new(move |_| {
        places
            .get_value()
            .into_iter()
            .filter(|p| {
                (p.field("season") == "All seasons"
                    || p.field("season") == season.get().label())
                    && (region.get() == "All regions" || p.field("region") == region.get())
                    && format!(
                        "{} {} {} {}",
                        p.title,
                        p.summary,
                        p.field("region"),
                        p.field("activity")
                    )
                    .to_lowercase()
                    .contains(&query.get().to_lowercase())
            })
            .collect::<Vec<_>>()
    });
    view!{<ManagedIntro slug="explore"/><crate::explorer::TrailExplorer/><section class="container section explorer-more-places"><div class="section-heading"><div><span class="eyebrow">"MAKE YOUR OWN CONNECTION"</span><h2>{move||format!("More {} experiences.",season.get().key())}</h2></div></div><div class="filter-bar"><crate::explorer::ExplorerSeason/><label class="region-select"><span class="sr-only">"Region"</span><select prop:value=move||region.get() on:change=move|e|region.set(event_target_value(&e))><option>"All regions"</option>{regions.into_iter().map(|r|view!{<option>{r}</option>}).collect_view()}</select></label><label class="search-field"><Icon name="search"/><span class="sr-only">"Search places"</span><input type="search" placeholder="Find a place or activity" prop:value=move||query.get() on:input=move|e|query.set(event_target_value(&e))/></label></div><p class="results-count" aria-live="polite">{move||format!("{} places to begin",filtered.get().len())}</p><div class="three-grid places-results">{move||filtered.get().into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div><Show when=move||filtered.get().is_empty()><div class="empty-state"><h2>"A different path, perhaps?"</h2><p>"Try another season, region, or search."</p><button class="button button-green" on:click=move |_|{region.set("All regions".into());query.set(String::new());}>"Clear filters"</button></div></Show></section>}.into_any()
}

#[component]
pub fn ManagedStories() -> impl IntoView {
    if catalog().is_none() {
        return view! {<pages::Stories/>}.into_any();
    }
    view!{<crate::editorial_pages::StoriesIndex/>}.into_any()
}

#[component]
pub fn Article(item: Content) -> impl IntoView {
    if item.kind == "campaign" && !item.field("project_status").is_empty() { return view!{<crate::support::ProjectDetail item/>}.into_any(); }
    if item.kind == "place" || item.kind == "story" { return view!{<crate::editorial::JournalArticle item/>}.into_any(); }
    let kind = item.kind.clone();
    let data = catalog().unwrap_or_default();
    let resource_url = if data.preview_id == Some(item.id) {
        url(&item.field("url"))
    } else {
        format!("{}{}", data.download_base, item.id)
    };
    view! {<ContentMeta item=item.clone()/><header class="article-header container"><p class="eyebrow"><span class="small-line"/>{item.field("category")}</p><h1>{item.title.clone()}</h1><p>{item.summary.clone()}</p></header>
        {(!item.field("image").is_empty()).then(||view!{<figure class="article-photo"><img src=item.image() style=item.image_position() alt=item.field("image_alt")/><figcaption>{item.field("credit")}</figcaption></figure>})}
        <div class="container article-grid section"><article class="prose"><BodyText text=item.body.clone()/>
            {(!item.field("url").is_empty()&&kind!="resource").then(||view!{<a class="text-link" href=url(&item.field("url")) target="_blank" rel="noopener noreferrer">{if kind=="place"{"Official visitor information"}else{"Continue with the original resources"}}<Icon name="external"/></a>})}
            {(!item.field("button_url").is_empty()).then(||view!{<a class="button button-green" href=url(&item.field("button_url"))>{item.field("button_text")}<Icon/></a>})}
            {(kind=="resource").then(||view!{<a class="button button-green" href=resource_url>"Open resource"<Icon name="external"/></a>})}
        </article><aside class="visit-card"><p class="eyebrow">"THE LIVING TRAIL"</p><h3>"Make a connection."</h3>
            {(kind=="place").then(||view!{<dl><div><dt>"Region"</dt><dd>{item.field("region")}</dd></div><div><dt>"Explore in"</dt><dd>{item.field("season")}</dd></div><div><dt>"Activities"</dt><dd>{item.field("activity")}</dd></div></dl><p class="form-note">"Confirm current conditions, access, and local guidance before you travel."</p>})}
            {(kind=="event").then(||view!{<dl><div><dt>"Starts (UTC)"</dt><dd>{item.field("start")}</dd></div><div><dt>"Ends (UTC)"</dt><dd>{item.field("end")}</dd></div><div><dt>"Location"</dt><dd>{item.field("location")}</dd></div></dl>})}
            <a class="text-link" href=url("/resources")>"Visitor & learning resources"<Icon/></a>
        </aside></div>
    }.into_any()
}

#[component]
pub fn ManagedDetail(kind: &'static str) -> impl IntoView {
    let params = use_params_map();
    if catalog().is_none() {
        return if kind == "place" {
            view! {<pages::PlaceDetail/>}.into_any()
        } else if kind == "story" {
            view! {<pages::StoryDetail/>}.into_any()
        } else {
            view! {<pages::NotFound/>}.into_any()
        };
    }
    view!{{move||{let slug=params.get().get("slug").unwrap_or_default();match items(kind).into_iter().find(|i|i.slug==slug){Some(item)=>view!{<Article item/>}.into_any(),None=>view!{<Missing/>}.into_any()}}}}.into_any()
}
#[component]
pub fn ManagedAbout() -> impl IntoView {
    if catalog().is_none() {
        return view! {<pages::About/>}.into_any();
    }
    match page("about") {
        Some(_) => view! {<crate::editorial_pages::AlliancePage/>}.into_any(),
        None => view! {<Missing/>}.into_any(),
    }
}

#[component]
pub fn ManagedResources() -> impl IntoView {
    if catalog().is_none() {
        return view! {<pages::Resources/>}.into_any();
    }
    let resources = StoredValue::new(items("resource"));
    let query = RwSignal::new(String::new());
    let category = RwSignal::new("All resources".to_string());
    let mut categories: Vec<_> = resources
        .get_value()
        .iter()
        .map(|i| i.field("category"))
        .filter(|s| !s.is_empty())
        .collect();
    categories.sort();
    categories.dedup();
    categories.insert(0, "All resources".into());
    let base = StoredValue::new(catalog().unwrap_or_default().download_base);
    let filtered = Memo::new(move |_| {
        resources
            .get_value()
            .into_iter()
            .filter(|i| {
                (category.get() == "All resources" || i.field("category") == category.get())
                    && format!("{} {}", i.title, i.summary)
                        .to_lowercase()
                        .contains(&query.get().to_lowercase())
            })
            .collect::<Vec<_>>()
    });
    view!{<ManagedIntro slug="resources" show_photo=false/><crate::editorial_pages::ResourcePrelude/><section class="container resource-library"><div class="resource-search"><Icon name="search"/><label class="sr-only" for="resource-query">"Search resources"</label><input id="resource-query" type="search" placeholder="Search guides, history, classroom resources…" prop:value=move||query.get() on:input=move|e|query.set(event_target_value(&e))/></div><div class="filter-pills resource-filters" role="group" aria-label="Resource categories">{categories.into_iter().map(|c|{let choice=c.clone();let active=c.clone();let pressed=c.clone();view!{<button class:active=move||category.get()==active aria-pressed=move||(category.get()==pressed).to_string() on:click=move |_|category.set(choice.clone())>{c}</button>}}).collect_view()}</div><p class="results-count" aria-live="polite">{move||format!("{} resources",filtered.get().len())}</p><div class="resource-list">{move||filtered.get().into_iter().map(|i|view!{<a href=format!("{}{}",base.get_value(),i.id) class="resource-row" target="_blank" rel="noopener noreferrer"><span class="resource-icon"><Icon name="book"/></span><div><span class="eyebrow">{i.field("category")}</span><h3>{i.title.clone()}</h3><p>{i.summary.clone()}</p></div><span class="file-type">{i.field("format")}</span><Icon name="external"/></a>}).collect_view()}</div><Show when=move||filtered.get().is_empty()><div class="empty-state"><h2>"Let's try another trail."</h2><p>"No resources match that search."</p><button class="button button-green" on:click=move |_|{query.set(String::new());category.set("All resources".into());}>"Clear search"</button></div></Show></section>}.into_any()
}

#[component]
pub fn ManagedSupport() -> impl IntoView {
    if catalog().is_none() {
        return view! {<pages::Support/>}.into_any();
    }
    view!{<crate::editorial_pages::SupportPage/>}.into_any()
}
#[component]
pub fn ManagedGeneric() -> impl IntoView {
    view! {<ManagedDetail kind="page"/>}
}
#[component]
pub fn Missing() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(response) = use_context::<leptos_axum::ResponseOptions>() {
        response.set_status(axum::http::StatusCode::NOT_FOUND);
    }
    view! {<pages::NotFound/>}
}
#[component]
pub fn CmsPreview() -> impl IntoView {
    let c = catalog().unwrap_or_default();
    let item = c.items.into_iter().find(|i| Some(i.id) == c.preview_id);
    view! {<Meta name="robots" content="noindex, nofollow"/><div class="cms-preview-banner">"PRIVATE DRAFT PREVIEW · Your public website has not changed."</div>{match item {
        Some(i) if i.kind=="page"=>match i.slug.as_str(){
            "home"=>view!{<crate::editorial::EditorialHome/>}.into_any(),"explore"=>view!{<ManagedExplore/>}.into_any(),"stories"=>view!{<ManagedStories/>}.into_any(),"resources"=>view!{<ManagedResources/>}.into_any(),"get-involved"=>view!{<ManagedSupport/>}.into_any(),"store"=>view!{<crate::editorial_pages::TrailShop/>}.into_any(),"about"=>view!{<crate::editorial_pages::AlliancePage/>}.into_any(),_=>view!{<Article item=i/>}.into_any()},
        Some(i)=>view!{<Article item=i/>}.into_any(),None=>view!{<Missing/>}.into_any(),
    }}}
}
