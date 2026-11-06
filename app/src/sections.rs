//! The same approved components render published pages and private draft previews.
use crate::{cms::{self,BodyText}, components::Icon, paths::url, photography,section_model::{self,Section}};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn PageSections()->impl IntoView {
    let location=use_location();
    view!{{move||{
        let path=location.pathname.get();let catalog=cms::catalog().unwrap_or_default();
        let item=if path.starts_with("/cms-preview/"){catalog.items.into_iter().find(|i|Some(i.id)==catalog.preview_id)}else{catalog.items.into_iter().find(|i|i.path==path)};
        let sections=item.and_then(|i|section_model::parse(&i.field(section_model::FIELD)).ok()).unwrap_or_default().sections;
        let visible:Vec<_>=sections.into_iter().filter(|s|!s.hidden).collect();
        (!visible.is_empty()).then(||view!{<div class="approved-page-sections">{visible.into_iter().map(|section|view!{<ApprovedSection section/>}).collect_view()}</div>})
    }}}
}
#[component]
fn SectionPhoto(section:Section)->impl IntoView {
    let item=section.content();let historical=photography::by_file(&section.image).is_some_and(|p|p.historical);
    (!section.image.is_empty()).then(||view!{<figure class="approved-photo" class:archival=historical><img src=url(&section.image) srcset=photography::srcset(&item) sizes="(max-width: 760px) 100vw, 65vw" style=item.image_position() alt=section.alt.clone() loading="lazy" width="1200" height="800"/><figcaption><span>{historical.then(||view!{<strong class="archive-label">"ARCHIVAL PHOTOGRAPH"</strong>})}{section.caption.clone()}<small>{section.credit.clone()}</small></span>{(!section.source_url.is_empty()).then(||view!{<a href=url(&section.source_url) target="_blank" rel="noopener noreferrer">"Photo source & rights"<Icon name="external"/></a>})}</figcaption></figure>})
}
#[component]
pub fn ApprovedSection(section:Section)->impl IntoView {
    let has_text=section.kind!="photo";let has_photo=section.has_photo();
    view!{<section class=format!("approved-section approved-{}",section.kind) data-section-id=section.id class:photo-right=section.reverse><div class="container approved-section-inner">
        {has_photo.then(||view!{<SectionPhoto section=section.clone()/>})}
        {has_text.then(||view!{<div class="approved-copy">{(!section.eyebrow.is_empty()).then(||view!{<span class="eyebrow">{section.eyebrow.clone()}</span>})}<h2>{section.title.clone()}</h2><div class="prose"><BodyText text=section.body.clone()/></div>{(!section.button_url.is_empty()&&!section.button_text.is_empty()).then(||view!{<a class="button button-green" href=url(&section.button_url)>{section.button_text.clone()}<Icon/></a>})}</div>})}
    </div></section>}
}
