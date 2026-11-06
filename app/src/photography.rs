//! Shared, credited photographic collection used by the site and Trail Studio.
use crate::{cms::Content, components::Icon, paths::url};
use leptos::prelude::*;
use serde::Deserialize;
use std::sync::OnceLock;

pub const CATEGORIES: &[&str] = &["Communities", "History & buildings", "Stewardship", "Summer travel", "Winter landscapes", "Trail details"];

#[derive(Clone, Debug, Deserialize)]
pub struct Photo {
    pub name: String, pub file: String, pub small_file: String,
    pub caption: String, pub alt: String, pub credit: String,
    pub source: String, pub license: String, pub license_url: String, pub changes: String,
    #[serde(default)] pub category: String,
    #[serde(default)] pub historical: bool,
    #[serde(default)] pub dimensions: Vec<u32>,
    #[serde(default)] pub small_dimensions: Vec<u32>,
}
impl Photo {
    pub fn matches(&self, category: &str, query: &str) -> bool {
        (category.is_empty() || self.category == category)
            && format!("{} {} {} {}", self.caption, self.alt, self.credit, self.category).to_lowercase().contains(&query.to_lowercase())
    }
    pub fn srcset(&self) -> String {
        if self.small_dimensions.first().is_some_and(|w| *w > 0) && self.dimensions.first().is_some_and(|w| *w > self.small_dimensions[0]) {
            format!("{} {}w, {} {}w", url(&self.small_file), self.small_dimensions[0], url(&self.file), self.dimensions[0])
        } else { String::new() }
    }
    pub fn credit_link(&self) -> String { url(&format!("/credits#photo-{}",self.name)) }
}
pub fn collection() -> &'static [Photo] {
    static PHOTOS: OnceLock<Vec<Photo>> = OnceLock::new();
    PHOTOS.get_or_init(|| {
        let mut photos = Vec::new();
        for raw in [include_str!("../public/images/original-manifest.json"), include_str!("../public/images/editorial/manifest.json"), include_str!("../public/images/explorer/manifest.json"), include_str!("../public/images/library-heritage/manifest.json"), include_str!("../public/images/library-travel/manifest.json")] {
            photos.extend(serde_json::from_str::<Vec<Photo>>(raw).expect("valid photographic collection"));
        }
        for p in &mut photos {
            if p.category.is_empty() { p.category = match p.name.as_str() {
                "seward-waterfront-archive"|"iditarod-first-street-1910"|"knik-townsite-cabin" => "History & buildings",
                "seward-harbor"|"nome-bering-sea"|"unalakleet-aerial" => "Communities",
                "crow-pass-valley" => "Summer travel",
                "seward-trail-monument"|"trail-marker-sunset" => "Trail details",
                _ => "Winter landscapes",
            }.into(); }
            if ["seward-waterfront-archive","iditarod-first-street-1910"].contains(&p.name.as_str()) {p.historical=true;}
        }
        photos
    })
}
pub fn by_file(file: &str) -> Option<Photo> {
    let path=file.strip_prefix(crate::paths::BASE_PATH).unwrap_or(file);
    collection().iter().find(|p|p.file==path).cloned()
}
pub fn named(name: &str) -> Option<Photo> {collection().iter().find(|p|p.name==name).cloned()}
pub fn with_fallback(mut item:Content,name:&str)->Content {
    if item.field("image").is_empty(){if let Some(p)=named(name){for(k,v)in[("image",p.file),("image_alt",p.alt),("credit",p.credit),("photo_caption",p.caption)]{item.fields.insert(k.into(),v);}}}
    item
}
pub fn thumbnail(item: &Content) -> String {by_file(&item.field("image")).map(|p|url(&p.small_file)).unwrap_or_else(||item.image())}
pub fn srcset(item: &Content) -> String {by_file(&item.field("image")).map(|p|p.srcset()).unwrap_or_default()}

#[component]
pub fn PhotoAttribution(item: Content) -> impl IntoView {
    let asset=by_file(&item.field("image"));
    let caption=if item.field("photo_caption").trim().is_empty(){asset.as_ref().map(|p|p.caption.clone()).unwrap_or_default()}else{item.field("photo_caption")};
    let credit=if item.field("credit").trim().is_empty(){asset.as_ref().map(|p|p.credit.clone()).unwrap_or_default()}else{item.field("credit")};
    view!{<div class="photo-attribution"><span>{asset.as_ref().filter(|p|p.historical).map(|_|view!{<strong class="archive-label">"ARCHIVAL PHOTOGRAPH"</strong>})}{caption}<small>{credit}</small></span>{asset.map(|p|view!{<a href=p.credit_link()>"Source & license"<Icon name="external"/></a>})}</div>}
}

#[component]
pub fn LibraryFigure(name: String, #[prop(default="")] class: &'static str) -> impl IntoView {
    named(&name).map(|p|view!{<figure class=format!("library-figure {class}") class:archival=p.historical><a class="library-photo-open" href=url(&p.file) target="_blank" rel="noopener noreferrer" aria-label=format!("View photograph: {}",p.caption)><img src=url(&p.file) srcset=p.srcset() sizes="(max-width: 760px) 100vw, 65vw" alt=p.alt.clone() loading="lazy" width=p.dimensions.first().copied().unwrap_or(1200) height=p.dimensions.get(1).copied().unwrap_or(800)/></a><figcaption><span>{p.historical.then(||view!{<strong class="archive-label">"ARCHIVAL PHOTOGRAPH"</strong>})}{p.caption.clone()}<small>{p.credit.clone()}</small></span><a href=p.credit_link()>"Photo details"<Icon name="external"/></a></figcaption></figure>})
}

#[component]
pub fn PhotoLibrary() -> impl IntoView {
    let category=RwSignal::new(String::new());let query=RwSignal::new(String::new());
    let filtered=move||collection().iter().filter(|p|p.matches(&category.get(),&query.get())).cloned().collect::<Vec<_>>();
    view!{<section class="container photographic-library" id="explorer-photography" aria-label="Photographic library"><div class="photo-library-controls"><label><span>"Search the collection"</span><input type="search" placeholder="Place, subject, or photographer" prop:value=move||query.get() on:input=move|e|query.set(event_target_value(&e))/></label><label><span>"Photograph category"</span><select aria-label="Photograph category" prop:value=move||category.get() on:change=move|e|category.set(event_target_value(&e))><option value="">"All photographs"</option>{CATEGORIES.iter().map(|c|view!{<option value=*c>{*c}</option>}).collect_view()}</select></label><p role="status">{move||format!("{} photographs",filtered().len())}</p></div><div class="credits-gallery">{move||filtered().into_iter().map(|p|view!{<article class="credit-photo" id=format!("photo-{}",p.name)><a class="credit-preview" href=url(&p.file) target="_blank" rel="noopener noreferrer"><img src=url(&p.small_file) alt=p.alt.clone() loading="lazy" width="600" height="400" class:archival=p.historical/></a><span class="eyebrow">{p.category.clone()}{p.historical.then_some(" / ARCHIVE")}</span><h2>{p.caption.clone()}</h2><p>{p.credit.clone()}</p><div><a href=p.source.clone() target="_blank" rel="noopener noreferrer">"Original record"<Icon name="external"/></a><a href=p.license_url.clone() target="_blank" rel="noopener noreferrer">{p.license.clone()}</a></div><small>{p.changes.clone()}</small></article>}).collect_view()}</div><Show when=move||filtered().is_empty()><div class="empty-state"><h2>"Try another view."</h2><p>"Search a different place or choose another collection."</p><button class="button button-green" on:click=move|_|{category.set(String::new());query.set(String::new());}>"Show all photographs"</button></div></Show></section>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collection_records_have_assets_rights_and_unique_names() {
        let mut names=std::collections::HashSet::new();let mut files=std::collections::HashSet::new();
        for p in collection(){
            assert!(names.insert(&p.name),"duplicate photo name: {}",p.name);assert!(files.insert(&p.file),"duplicate photo file: {}",p.file);
            for path in [&p.file,&p.small_file]{assert!(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public").join(path.trim_start_matches('/')).is_file(),"missing {path}");}
            assert!(p.source.starts_with("https://")&&p.license_url.starts_with("https://"));assert!(!p.credit.is_empty()&&!p.caption.is_empty()&&!p.alt.is_empty());
            assert!(CATEGORIES.contains(&p.category.as_str()));assert!(!p.license.contains("NC"),"collection must allow the proposed site use");
        }
        assert!(collection().len()>=30);
        assert!(collection().iter().any(|p|p.category=="Stewardship"));
    }
}
