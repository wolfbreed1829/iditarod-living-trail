//! Destination guides, photographic stories and the Alliance's institutional history.
use crate::{cms::{self, BodyText, Content, ContentCard, ContentMeta}, components::Icon, paths::url, photography::{self, LibraryFigure, PhotoAttribution}};
use leptos::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, sync::OnceLock};

#[derive(Clone, Deserialize, Default)]
struct Link {title:String, url:String}
#[derive(Clone, Deserialize, Default)]
struct Experience {title:String, copy:String, url:String}
#[derive(Clone, Deserialize, Default)]
struct SourceSection {title:String, copy:String, source:String, #[serde(default)] date:String}
#[derive(Clone, Deserialize, Default)]
struct Guide {heading:String, orientation:String, experiences:Vec<Experience>, planning:Vec<String>, #[serde(default)] history:String, sources:Vec<String>, map:String, gallery:Vec<String>}
#[derive(Clone, Deserialize, Default)]
struct Story {sections:Vec<SourceSection>, links:Vec<Link>, map:String, map_note:String, gallery:Vec<String>}
#[derive(Clone, Deserialize, Default)]
struct Alliance {intro:String, timeline_heading:String, timeline:Vec<SourceSection>, work_themes:Vec<SourceSection>}
#[derive(Deserialize)]
struct InteriorContent {destinations:HashMap<String,Guide>,stories:HashMap<String,Story>,alliance:Alliance}
fn content()-> &'static InteriorContent {
    static CONTENT:OnceLock<InteriorContent>=OnceLock::new();
    CONTENT.get_or_init(||serde_json::from_str(include_str!("../public/interior-content.json")).expect("valid interior content"))
}
fn gallery(item:&Content,defaults:Vec<String>)->Vec<String> {
    (0..2).filter_map(|i|{let choice=item.field(&format!("essay_photo_{}",i+1));if choice=="none"{None}else if choice.is_empty(){defaults.get(i).cloned()}else{Some(choice)}}).filter(|name|photography::named(name).is_some()).collect()
}

#[component]
fn ContentActions(item:Content, #[prop(default=false)] source:bool) -> impl IntoView {
    view!{<div class="interior-content-actions">{(source&&!item.field("url").is_empty()).then(||view!{<a class="text-link" href=url(&item.field("url")) target="_blank" rel="noopener noreferrer">"Read the original resources"<Icon name="external"/></a>})}{(!item.field("button_url").is_empty()).then(||view!{<a class="button button-green" href=url(&item.field("button_url"))>{item.field("button_text")}<Icon/></a>})}</div>}
}

#[component]
pub fn HeroPhoto(item:Content) -> impl IntoView {
    let historical=photography::by_file(&item.field("image")).is_some_and(|p|p.historical);
    (!item.field("image").is_empty()).then(||view!{<figure class="journal-article-photo interior-hero-photo" class:photo-tall=item.field("image_layout")=="tall" class:archival=historical><img src=item.image() srcset=photography::srcset(&item) sizes="100vw" style=item.image_position() alt=item.field("image_alt") fetchpriority="high" width="1600" height="1000"/><figcaption><PhotoAttribution item=item.clone()/></figcaption></figure>})
}

#[component]
pub fn SupportingMap(place:String) -> impl IntoView {
    let all=crate::explorer::stops_with_content(vec![]);
    let regional=["seward","crow-pass","knik"].contains(&place.as_str());
    let selected=all.iter().find(|p|p.id==place).cloned();
    let pins:Vec<_>=if let Some(p)=selected.clone(){vec![p]}else{all.into_iter().filter(|p|p.id=="seward"||p.id=="nome").collect()};
    let heading=selected.as_ref().map(|p|p.name.clone()).unwrap_or("Seward to Nome".into());
    let destination=selected.as_ref().map(|p|format!("/explore?place={}#trail-atlas",p.id)).unwrap_or("/explore#trail-atlas".into());
    let svg=include_str!("../public/maps/explorer-base.svg");
    let geometry=svg[svg.find('>').unwrap()+1..svg.rfind("</svg>").unwrap()].replace("explorer-base-","interior-base-");
    view!{<figure class="supporting-map"><div class="supporting-map-heading"><span>"THE GEOGRAPHIC CONNECTION"</span><h3>{heading}</h3></div><div class="interior-map-canvas" class:regional=regional><svg class="interior-map-svg explorer-geography" fill="none" viewBox=if regional{"590 420 240 186.6667"}else{"0 0 900 700"} role="img" aria-label=if regional{"Historic trail orientation in Southcentral Alaska"}else{"Main historic trail alignment between Seward and Nome in Alaska"} inner_html=geometry/>{pins.into_iter().map(|p|{let(x,y,w,h)=if regional{(590.,420.,240.,186.6667)}else{(0.,0.,900.,700.)};view!{<span class="interior-map-pin" style=format!("left:{}%;top:{}%",(p.x-x)/w*100.,(p.y-y)/h*100.)><i/><span>{p.name}</span></span>}}).collect_view()}</div><figcaption><span>"Historic main route · Orientation only"</span><a href=url("/credits#map-sources")>"Map sources"</a></figcaption><a class="supporting-map-action" href=url(&destination)>"Open the interactive explorer"<Icon/></a></figure>}
}

#[component]
fn Related(item:Content) -> impl IntoView {
    let related:Vec<_>=cms::items(&item.kind).into_iter().filter(|i|i.id!=item.id).take(2).collect();
    (!related.is_empty()).then(||view!{<section class="journal-related interior-related"><div class="container"><div class="section-heading"><div><span class="eyebrow">"KEEP FOLLOWING YOUR CURIOSITY"</span><h2>{if item.kind=="place"{"Another place to begin."}else{"The story continues."}}</h2></div><a class="text-link" href=url(if item.kind=="place"{"/explore"}else{"/stories"})>"View all"<Icon/></a></div><div class="two-grid">{related.into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div></div></section>})
}

#[component]
pub fn DestinationGuide(item:Content) -> impl IntoView {
    let guide=content().destinations.get(&item.slug).cloned().unwrap_or_default();
    let map=if item.field("map_stop").is_empty(){guide.map.clone()}else{item.field("map_stop")};
    let has_experiences=!guide.experiences.is_empty();
    let intro=if item.field("visit_notes").trim().is_empty(){item.body.clone()}else{item.field("visit_notes")};
    let heading=if guide.heading.is_empty(){"Find your way into the place.".into()}else{guide.heading.clone()};
    let mut planning=guide.planning.clone();
    for(index,key)in[(0,"season_notes"),(1,"access_notes")]{let value=item.field(key);if !value.trim().is_empty(){if planning.len()>index{planning[index]=value;}else{planning.push(value);}}}
    if planning.is_empty(){planning.push("Use the official visitor information for current access, seasonal conditions, and local guidance.".into());}
    let photos=guide.gallery.clone();
    view!{<ContentMeta item=item.clone()/><div class="destination-guide"><div class="container breadcrumb"><a href=url("/explore")>"Explore the trail"</a><span>"/"</span><span>{item.field("region")}</span></div><header class="container destination-heading"><div><span class="eyebrow">"A PLACE ON THE LIVING TRAIL"</span><h1>{item.title.clone()}</h1></div><div><p>{item.summary.clone()}</p><span class="destination-location"><Icon name="pin"/>{item.field("region")}</span></div></header><HeroPhoto item=item.clone()/><nav class="interior-section-nav" aria-label="Destination guide sections"><div class="container"><a href="#place-overview">"The place"</a>{has_experiences.then(||view!{<a href="#place-experiences">"Ways to explore"</a>})}<a href="#place-planning">"Plan your visit"</a><a href="#place-map">"On the map"</a></div></nav>
        <section class="container destination-overview" id="place-overview"><div><span class="section-marker"><span>"01"</span>"GET TO KNOW THE PLACE"</span><h2>{heading}</h2><div class="prose"><BodyText text=intro/><ContentActions item=item.clone()/></div></div><aside class="destination-fieldnotes" aria-label="Destination field notes"><span class="eyebrow">"AT A GLANCE"</span><dl>{[("Setting",item.field("region")),("Season",item.field("season")),("Ways to explore",item.field("activity"))].into_iter().filter(|(_,v)|!v.is_empty()).map(|(k,v)|view!{<div><dt>{k}</dt><dd>{v}</dd></div>}).collect_view()}</dl><p>{guide.orientation.clone()}</p>{(!item.field("url").is_empty()).then(||view!{<a class="text-link" href=url(&item.field("url")) target="_blank" rel="noopener noreferrer">"Official visitor information"<Icon name="external"/></a>})}</aside></section>
        {has_experiences.then(||view!{<section class="destination-experiences" id="place-experiences"><div class="container"><div class="section-heading"><div><span class="section-marker"><span>"02"</span>"MAKE A CONNECTION"</span><h2>"More than a place to pass through."</h2></div><p>"Choose an experience that fits your time, your curiosity, and the season."</p></div><div class="destination-experience-grid">{guide.experiences.into_iter().enumerate().map(|(i,e)|view!{<article><span>{format!("0{}",i+1)}</span><h3>{e.title}</h3><p>{e.copy}</p><a class="text-link" href=url(&e.url) target=if e.url.starts_with("http"){"_blank"}else{"_self"} rel="noopener noreferrer">"Follow this idea"<Icon/></a></article>}).collect_view()}</div></div></section>})}
        {(!photos.is_empty()).then(||view!{<section class="container destination-photo-sequence" aria-label="Photographs of the place"><div class="section-heading"><span class="eyebrow">"LOOK A LITTLE CLOSER"</span><h2>"Details that stay with you."</h2></div><div class="photo-sequence-grid">{photos.into_iter().map(|name|view!{<LibraryFigure name/>}).collect_view()}</div></section>})}
        <section class="container destination-planning" id="place-planning"><div><span class="section-marker"><span>"03"</span>"BEFORE YOU GO"</span><h2>"A little preparation."<br/><em>"A better connection."</em></h2><div class="planning-notes">{planning.into_iter().enumerate().map(|(i,p)|view!{<div><span>{format!("0{}",i+1)}</span><p>{p}</p></div>}).collect_view()}</div>{(!guide.history.is_empty()).then(||view!{<div class="destination-history"><span class="eyebrow">"A PLACE IN HISTORY"</span><p>{guide.history.clone()}</p></div>})}<div class="guide-sources"><span>"Planning sources"</span>{guide.sources.into_iter().enumerate().map(|(i,href)|view!{<a href=href target="_blank" rel="noopener noreferrer">{format!("Source {}",i+1)}<Icon name="external"/></a>}).collect_view()}</div></div><aside id="place-map"><SupportingMap place=map/><p class="map-planning-note">"Use the map to understand the connection. Check local access and current conditions before traveling."</p></aside></section><Related item/></div>}
}

#[component]
pub fn MagazineStory(item:Content) -> impl IntoView {
    let story=content().stories.get(&item.slug).cloned().unwrap_or_default();
    let photos=gallery(&item,story.gallery.clone());
    let map=match item.field("story_map").as_str(){""=>story.map.clone(),other=>other.to_string()};
    let words=item.body.split_whitespace().count();
    let paragraphs:Vec<_>=item.body.split("\n\n").filter(|p|!p.trim().is_empty()).map(str::to_owned).collect();
    let remaining=photos.iter().skip(paragraphs.len()).cloned().collect::<Vec<_>>();
    let sections=story.sections.clone();let photos=StoredValue::new(photos);
    let sources=story.links.clone();
    view!{<ContentMeta item=item.clone()/><div class="magazine-story"><div class="container breadcrumb"><a href=url("/stories")>"The trail journal"</a><span>"/"</span><span>{item.field("category")}</span></div><header class="container magazine-heading"><span class="eyebrow">"PEOPLE / PLACE / CONTINUITY"</span><h1>{item.title.clone()}</h1><div><p>{item.summary.clone()}</p><span class="magazine-reading">{format!("{} MIN READ",words.div_ceil(200).max(1))}<span>"THE LIVING TRAIL JOURNAL"</span></span></div></header><HeroPhoto item=item.clone()/><div class="container magazine-layout"><article class="magazine-prose" aria-label="Story"><div class="magazine-story-label"><span>"THE STORY"</span><span>"01"</span></div>{paragraphs.into_iter().enumerate().map(|(i,text)|{let section=sections.get(i).cloned();view!{<section class="magazine-chapter" id=format!("story-section-{}",i+1)>{section.as_ref().map(|s|view!{<h2>{s.title.clone()}</h2>})}<BodyText text/>{section.map(|s|view!{<a class="story-source-note" href=s.source target="_blank" rel="noopener noreferrer">"Read the source"<Icon name="external"/></a>})}</section>{photos.get_value().get(i).cloned().map(|name|view!{<LibraryFigure name class="magazine-sequence-photo"/>})}}}).collect_view()}{remaining.into_iter().map(|name|view!{<LibraryFigure name class="magazine-sequence-photo"/>}).collect_view()}<ContentActions item=item.clone() source=true/></article><aside class="magazine-sidebar" aria-label="Story context"><div class="magazine-sidebar-inner"><span class="eyebrow">"FOLLOW THE CONNECTION"</span>{(!map.is_empty()&&map!="none").then(||view!{<SupportingMap place=map/>})}<p>{story.map_note}</p><div class="magazine-note"><h2>"Look closer. Listen longer."</h2><p>"The trail's history lives in places, records, and the voices of the people who know it."</p><a class="text-link" href=url("/resources")>"Open the field library"<Icon/></a></div></div></aside></div>
        {(!sources.is_empty()).then(||view!{<section class="story-further-reading"><div class="container"><span class="eyebrow">"TAKE THE STORY FURTHER"</span><h2>"A starting point for your own discovery."</h2><div>{sources.into_iter().enumerate().map(|(i,s)|view!{<a href=s.url target="_blank" rel="noopener noreferrer"><span>{format!("0{}",i+1)}</span><h3>{s.title}</h3><Icon name="external"/></a>}).collect_view()}</div></div></section>})}<Related item/></div>}
}

#[component]
pub fn AllianceHistory(item:Content) -> impl IntoView {
    let alliance=content().alliance.clone();
    view!{<ContentMeta item=item.clone()/><div class="alliance-editorial"><header class="container alliance-heading"><span class="eyebrow">"THE IDITAROD HISTORIC TRAIL ALLIANCE"</span><h1>{item.title.clone()}</h1><p>{item.summary.clone()}</p></header><HeroPhoto item=item.clone()/><nav class="interior-section-nav" aria-label="Alliance page sections"><div class="container"><a href="#alliance-purpose">"Our purpose"</a><a href="#alliance-history">"Our history"</a><a href="#alliance-work">"Preservation at work"</a></div></nav><section class="container alliance-story" id="alliance-purpose"><div><span class="section-marker"><span>"01"</span>"A SHARED RESPONSIBILITY"</span><h2>"A living trail."<br/><em>"A continuing promise."</em></h2></div><div class="prose"><BodyText text=item.body.clone()/><p>{alliance.intro}</p><ContentActions item=item.clone()/></div></section>
        <section class="alliance-history-section" id="alliance-history"><div class="container"><div class="section-heading"><div><span class="section-marker"><span>"02"</span>"THE LONG VIEW"</span><h2>{alliance.timeline_heading}</h2></div><p>"The trail's story begins long before a designation or an organization. These milestones trace the work of recognizing and caring for that heritage."</p></div><div class="alliance-history-grid"><div class="alliance-timeline">{alliance.timeline.into_iter().map(|e|view!{<article><span>{e.date}</span><div><h3>{e.title}</h3><p>{e.copy}</p><a href=e.source target="_blank" rel="noopener noreferrer">"Read the record"<Icon name="external"/></a></div></article>}).collect_view()}</div><div class="alliance-history-photo"><LibraryFigure name="safety-roadhouse-habs".to_string()/><div class="history-pullout"><span class="eyebrow">"PLACES HOLD MEMORY"</span><p>"A stopping place. A shared route. A story carried forward."</p></div><LibraryFigure name="knik-hall-2009".to_string()/></div></div></div></section>
        <section class="container alliance-preservation" id="alliance-work"><div class="section-heading"><div><span class="section-marker"><span>"03"</span>"PRESERVATION IN PRACTICE"</span><h2>"Care takes many forms."</h2></div><p>"On the ground, in the archive, and in the classroom, the work connects people to the trail's future."</p></div><div class="preservation-work-grid">{alliance.work_themes.into_iter().enumerate().map(|(i,w)|view!{<article><span>{format!("0{}",i+1)}</span><h3>{w.title}</h3><p>{w.copy}</p><a class="text-link" href=w.source target="_blank" rel="noopener noreferrer">"Explore this work"<Icon name="external"/></a></article>}).collect_view()}</div><div class="alliance-work-photo"><LibraryFigure name="unalakleet-fish-house-2013".to_string()/><div><span class="eyebrow">"COMMUNITIES ALONG THE WAY"</span><h2>"The connection is human."</h2><p>"The trail crosses a lived-in landscape. Respect for local knowledge, land ownership, and community histories belongs at the center of its care."</p><a class="text-link" href=url("/explore")>"Meet the places along the trail"<Icon/></a></div></div><a class="text-link" href="https://www.iditarod100.org/annual-reports.html" target="_blank" rel="noopener noreferrer">"Read the Alliance's annual reports"<Icon name="external"/></a></section><section class="alliance-join"><div class="container"><span class="eyebrow">"THE NEXT CHAPTER"</span><h2>"Help carry it forward."</h2><p>"Bring your curiosity, your time, and your care for the places that connect us."</p><a class="button button-green" href=url("/get-involved")>"Find your part"<Icon/></a></div></section></div>}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn supporting_photographs_and_maps_resolve_to_verified_collection_entries(){
        for g in content().destinations.values(){for name in &g.gallery{assert!(photography::named(name).is_some(),"missing {name}");}assert!(g.map=="whole-trail"||crate::explorer::STOPS.iter().any(|(id,_)|*id==g.map));}
        for s in content().stories.values(){for name in &s.gallery{assert!(photography::named(name).is_some(),"missing {name}");}}
    }
    #[test]
    fn staff_can_replace_or_remove_editorial_photos_without_changing_the_story(){
        let mut item=Content::default();item.fields.insert("essay_photo_1".into(),"none".into());item.fields.insert("essay_photo_2".into(),"seward-harbor".into());
        assert_eq!(gallery(&item,vec!["dawn".into(),"dogs".into()]),vec!["seward-harbor"]);
    }
}
