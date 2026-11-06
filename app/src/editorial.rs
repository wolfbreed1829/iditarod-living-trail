//! Editorial presentation for the living, CMS-managed trail.
use crate::{cms::{self, Content, ContentMeta, BodyText, ContentCard}, components::*, motion::{Reveal, SeasonalHero}, paths::url};
use leptos::prelude::*;
use leptos_meta::Title;

fn photo(item: &Content) -> String {
    if item.field("image").is_empty() { url("/images/dawn.webp") } else { item.image() }
}
fn position(item: &Content) -> String {
    let x = item.field("image_x").parse::<u8>().unwrap_or(50).min(100);
    let y = item.field("image_y").parse::<u8>().unwrap_or(50).min(100);
    format!("object-position:{x}% {y}%")
}
fn reading_time(item: &Content) -> usize { item.body.split_whitespace().count().div_ceil(200).max(1) }

#[component]
pub fn EditorialHome() -> impl IntoView {
    let paused = RwSignal::new(false);
    let home = cms::page("home").unwrap_or_default();
    let home_body = home.body.clone();
    view! {
        <Title text="Iditarod Historic Trail Alliance | A living trail through Alaska"/>
        <ContentMeta item=home/>
        <div class="motion-scene editorial-home" class:motion-paused=move || paused.get()>
            <SeasonalHero paused/>
            <div class="trail-index"><div class="container"><span class="index-label">"THE IDITAROD NATIONAL HISTORIC TRAIL"</span><span>"Seward"<span class="index-route" aria-hidden="true"/>"Nome"</span><a href="#trail-atlas">"Follow the connection"<Icon name="down"/></a></div></div>
            <section class="journal-opening container" id="discover">
                <Reveal><div class="section-marker"><span>"01"</span>"A PLACE IN THE STORY"</div><div class="opening-grid"><h2>"One trail."<br/><em>"An Alaska of"<br/>"possibilities."</em></h2><div class="opening-copy"><p class="lead">"A path through extraordinary landscapes. A connection between people, places, and generations."</p><p>"The Iditarod is a living network of winter trails, community connections, and shared history. There are many ways to find your place in its story."</p><a class="text-link" href=url("/about")>"Meet the Alliance"<Icon/></a></div></div></Reveal>
                {(!home_body.is_empty()).then(||view!{<div class="prose cms-home-intro"><BodyText text=home_body/></div>})}
                <div class="journal-pathways">
                    <a class="pathway-major" href=url("/explore")><img src=url("/images/summer.webp") alt="Glacier and mountain landscape in Chugach National Forest" loading="lazy" width="1600" height="1100"/><div class="photo-scrim"/><span class="pathway-number">"01 / EXPERIENCE"</span><div><h3>"Go a little"<br/><em>"further."</em></h3><span class="pathway-link">"Explore places along the trail"<Icon/></span></div></a>
                    <div class="pathway-side"><a class="pathway-small" href=url("/stories")><div><span class="eyebrow">"02 / DISCOVER"</span><h3>"History with"<br/><em>"a heartbeat."</em></h3><span class="text-link">"Stories & heritage"<Icon/></span></div><img src=url("/images/editorial/seward-waterfront-archive-small.webp") alt="Historic Seward waterfront before the 1964 earthquake" loading="lazy" width="800" height="600"/></a><a class="pathway-note" href=url("/get-involved")><span class="eyebrow">"03 / CARRY IT FORWARD"</span><h3>"A lasting trail."<br/><em>"A shared responsibility."</em></h3><p>"Help preserve its history, care for its places, and connect the next generation."</p><span class="text-link">"Make a difference"<Icon/></span><span class="note-mark" aria-hidden="true"><Icon name="mountain"/></span></a></div>
                </div>
            </section>
            <TrailAtlas/>
            <section class="journal-places container section"><Reveal><div class="section-heading"><div><span class="section-marker"><span>"03"</span>"PLACES TO BEGIN"</span><h2>"Find your own"<br/><em>"way into Alaska."</em></h2></div><div class="heading-aside"><p>"Coastal communities. Mountain country. The quiet of a winter trail."</p><a class="text-link" href=url("/explore")>"Explore the trail"<Icon/></a></div></div></Reveal><cms::ManagedHomePlaces/></section>
            <JournalStories/>
            <section class="journal-invitation"><div class="container"><span class="eyebrow">"THE NEXT CHAPTER"</span><div><h2>"Some things are"<br/><em>"worth carrying forward."</em></h2><div><p>"The trail's future is a shared undertaking. Bring your curiosity, your time, or your support."</p><a class="button button-green" href=url("/get-involved")>"Be part of the trail"<Icon/></a></div></div><span class="invitation-rule"/></div></section>
        </div>
    }
}

#[component]
pub fn JournalStories() -> impl IntoView {
    let stories = cms::items("story");
    let featured = stories.iter().find(|s|s.slug=="a-living-connection").or(stories.first()).cloned();
    let rest: Vec<_> = stories.iter().filter(|s| featured.as_ref().is_none_or(|f|f.id!=s.id)).cloned().collect();
    view! {<section class="journal-stories"><div class="container"><div class="section-heading"><div><span class="section-marker"><span>"04"</span>"THE TRAIL JOURNAL"</span><h2>"Every mile"<br/><em>"carries a story."</em></h2></div><a class="text-link" href=url("/stories")>"All stories"<Icon/></a></div><div class="journal-stories-grid">
        {featured.map(|s|view!{<a class="journal-lead-story" href=url(&s.path)><div class="journal-story-image"><img src=photo(&s) style=position(&s) alt=s.field("image_alt") loading="lazy" width="1200" height="800"/><span class="journal-image-label">"THE LIVING TRAIL"</span></div><div><span class="eyebrow">{s.field("category")}</span><h3>{s.title.clone()}</h3><p>{s.summary.clone()}</p><span class="text-link">"Read the story"<Icon/></span></div></a>})}
        <div class="journal-story-list">{rest.into_iter().enumerate().map(|(index,s)|view!{<a class="journal-story-row" href=url(&s.path)><span class="story-order">{format!("0{}",index+2)}</span><div><span class="eyebrow">{s.field("category")}</span><h3>{s.title.clone()}</h3><p>{s.summary.clone()}</p><span class="story-reading">{format!("{} min read",reading_time(&s))}<Icon/></span></div><img src=photo(&s) style=position(&s) alt=s.field("image_alt") loading="lazy" width="320" height="380"/></a>}).collect_view()}<a class="journal-archive-link" href=url("/resources")><Icon name="book"/><div><span class="eyebrow">"A LITTLE DEEPER"</span><h3>"Open the archive."</h3><p>"Maps, history, and resources for curious minds."</p></div><Icon/></a></div>
    </div></div></section>}
}

#[derive(Clone, Copy)]
struct TrailPlace { name: &'static str, region: &'static str, title: &'static str, text: &'static str, image: &'static str, alt: &'static str, link: &'static str, x: f32, y: f32 }
const ATLAS_PLACES: [TrailPlace; 4] = [
    TrailPlace{name:"Seward",region:"01 / THE SOUTHERN GATEWAY",title:"Where the mountains meet the sea.",text:"On Resurrection Bay, the southern gateway connects Alaska's coast to the Iditarod trail's inland history.",image:"/images/editorial/seward-harbor.webp",alt:"Boats and mountain scenery at Seward harbor",link:"/places/seward",x:77.71,y:83.36},
    TrailPlace{name:"Southcentral",region:"02 / MOUNTAIN COUNTRY",title:"Into Chugach country.",text:"Glaciers, forests, and mountain passes shape the southern trail. Discover the landscape, then plan with current local guidance.",image:"/images/summer.webp",alt:"Glacier and mountain landscape in Chugach National Forest",link:"/places/chugach",x:78.31,y:74.68},
    TrailPlace{name:"Interior",region:"03 / A LIVING CONNECTION",title:"Stories carried across a landscape.",text:"Alaska Native travel networks, community connections, and winter journeys are woven into the trail's history. Pictured: Old Woman cabin, farther along the trail toward Unalakleet.",image:"/images/editorial/old-woman-cabin.webp",alt:"Old Woman shelter cabin on the Iditarod National Historic Trail",link:"/stories/a-living-connection",x:44.76,y:58.52},
    TrailPlace{name:"Nome",region:"04 / THE BERING SEA COAST",title:"The journey becomes a legacy.",text:"Nome is the northern gateway of the historic trail. Its coastal setting is part of a much wider story of people and place.",image:"/images/editorial/nome-bering-sea.webp",alt:"Bering Sea shoreline near Nome, Alaska, with offshore sea ice",link:"/stories",x:20.89,y:34.76},
];

#[component]
pub fn TrailAtlas() -> impl IntoView {
    let selected = RwSignal::new(0usize);
    view!{<section class="trail-atlas" id="trail-atlas" aria-labelledby="atlas-title"><div class="container"><div class="atlas-heading"><div><span class="section-marker"><span>"02"</span>"FOLLOW THE CONNECTION"</span><h2 id="atlas-title">"A landscape of"<br/><em>"living connections."</em></h2></div><p>"Explore four chapters of the historic trail, from the Gulf of Alaska to the Bering Sea."</p></div><div class="atlas-grid"><div class="atlas-map"><div class="map-overline"><span>"ALASKA / THE HISTORIC TRAIL"</span><span>"NORTH"</span></div><div class="atlas-map-canvas"><img class="atlas-basemap" src=url("/maps/trail-orientation.svg") alt="Geographic overview of the Iditarod National Historic Trail in Alaska" width="900" height="700"/>{ATLAS_PLACES.into_iter().enumerate().map(|(i,p)|view!{<button class="atlas-pin" class:active=move||selected.get()==i style=format!("left:{}%;top:{}%",p.x,p.y) aria-label=format!("Explore {}",p.name) aria-pressed=move||(selected.get()==i).to_string() on:click=move|_|selected.set(i)><span class="pin-dot"/><span class="pin-label">{p.name}</span></button>}).collect_view()}</div><div class="atlas-map-footer"><span><i/>"Historic trail alignment"</span><a href=url("/credits#map-sources")>"Map sources"<Icon name="external"/></a></div></div><div class="atlas-detail"><div class="chapter-controls" role="group" aria-label="Trail chapters">{ATLAS_PLACES.into_iter().enumerate().map(|(i,p)|view!{<button class:active=move||selected.get()==i aria-pressed=move||(selected.get()==i).to_string() aria-controls="trail-chapter" on:click=move|_|selected.set(i)><span>{format!("0{}",i+1)}</span>{p.name}</button>}).collect_view()}</div><div id="trail-chapter" aria-live="polite" aria-atomic="true">{move||{let p=ATLAS_PLACES[selected.get()];view!{<div class="atlas-photo"><img src=url(p.image) alt=p.alt loading="lazy" width="900" height="620"/></div><span class="eyebrow">{p.region}</span><h3>{p.title}</h3><p>{p.text}</p><a class="text-link" href=url(p.link)>"Discover this chapter"<Icon/></a>}}}</div></div></div><p class="atlas-note">"Historic trail overview / BLM route, Natural Earth coastline. For orientation only; not a navigation or conditions map."</p></div></section>}
}

#[component]
pub fn JournalArticle(item: Content) -> impl IntoView {
    let is_place = item.kind == "place";
    let title = item.title.clone();
    let related:Vec<_> = cms::items(&item.kind).into_iter().filter(|i|i.id!=item.id).take(2).collect();
    let back = if is_place {"/explore"} else {"/stories"};
    let back_label = if is_place {"Explore the trail"} else {"The trail journal"};
    let category = item.field(if is_place {"region"} else {"category"});
    let label = if category.is_empty() {"THE LIVING TRAIL".to_string()} else {category};
    view!{<ContentMeta item=item.clone()/><div class="journal-article" class:place-article=is_place>
        <div class="container breadcrumb"><a href=url(back)>{back_label}</a><span>"/"</span><span>{title.clone()}</span></div>
        <header class="journal-article-header container"><span class="eyebrow"><span class="small-line"/>{label}</span><h1>{title}</h1><div class="article-deck"><p>{item.summary.clone()}</p><span class="article-meta">{if is_place {"A PLACE ON THE TRAIL".to_string()} else {format!("THE TRAIL JOURNAL / {} MIN READ",reading_time(&item))}}</span></div></header>
        {(!item.field("image").is_empty()).then(||view!{<figure class="journal-article-photo"><img src=photo(&item) style=position(&item) alt=item.field("image_alt") fetchpriority="high" width="1800" height="1100"/><figcaption><span>{item.field("credit")}</span><a href=url("/credits")>"Photography & sources"<Icon name="external"/></a></figcaption></figure>})}
        <div class="container journal-article-grid"><aside class="article-margin" aria-label="Article navigation"><span class="section-marker"><span>"01"</span>{if is_place {"EXPLORE THIS PLACE"} else {"THE STORY"}}</span><span class="margin-rule"/><a href=url(back) class="text-link">{back_label}<Icon/></a></aside><article class="prose journal-prose"><BodyText text=item.body.clone()/>
            {(!item.field("url").is_empty()).then(||view!{<a class="article-source" href=url(&item.field("url")) target="_blank" rel="noopener noreferrer"><span class="eyebrow">{if is_place {"BEFORE YOU GO"} else {"CONTINUE EXPLORING"}}</span><strong>{if is_place {"Check official visitor information"} else {"Read the original resources"}}</strong><Icon name="external"/></a>})}
            {(!item.field("button_url").is_empty()).then(||view!{<a class="button button-green" href=url(&item.field("button_url"))>{item.field("button_text")}<Icon/></a>})}
        </article><aside class="field-guide" aria-label="Trail field notes"><span class="eyebrow">{if is_place {"YOUR FIELD NOTES"} else {"PART OF A LIVING TRAIL"}}</span><h2>{if is_place {"At a glance."} else {"Keep the connection."}}</h2>
            {if is_place {view!{<dl><div><dt>"Region"</dt><dd>{item.field("region")}</dd></div><div><dt>"Season"</dt><dd>{item.field("season")}</dd></div><div><dt>"Ways to explore"</dt><dd>{item.field("activity")}</dd></div></dl><p>"Access and conditions change. Respect local communities, land ownership, and current guidance."</p>}.into_any()} else {view!{<p>"Explore the places behind the stories, and the work that carries the trail into its next chapter."</p><a class="text-link" href=url("/get-involved")>"Support the trail"<Icon/></a>}.into_any()}}
            <a class="text-link" href=url("/resources")>"Maps & resources"<Icon/></a>
        </aside></div>
        {(!related.is_empty()).then(||view!{<section class="journal-related"><div class="container"><div class="section-heading"><div><span class="eyebrow">"KEEP YOUR CURIOSITY"</span><h2>{if is_place {"A little further along."} else {"Another chapter awaits."}}</h2></div><a class="text-link" href=url(back)>"View all"<Icon/></a></div><div class="two-grid">{related.into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div></div></section>})}
    </div>}
}


#[component]
pub fn EditorialCredits() -> impl IntoView {
    let editorial: Vec<serde_json::Value> = serde_json::from_str(include_str!("../public/images/editorial/manifest.json")).unwrap_or_default();
    let original: Vec<serde_json::Value> = serde_json::from_str(include_str!("../public/image-sources.json")).unwrap_or_default();
    view!{<Title text="Photography & map sources | The Living Trail"/><section class="container page-intro"><span class="eyebrow">"THE PEOPLE BEHIND THE PICTURES"</span><h1>"Every image has a story."</h1><p class="intro-copy">"Real places, authentic photographs, and the people who help us see the trail differently."</p></section><section class="container credits-gallery">{editorial.into_iter().map(|asset|{
        let field = |k:&str|asset[k].as_str().unwrap_or_default().to_string();
        view!{<article class="credit-photo"><img src=url(&field("small_file")) alt=field("alt") loading="lazy" width="900" height="600"/><h2>{field("caption")}</h2><p>{field("credit")}</p><div><a href=field("source") target="_blank" rel="noopener noreferrer">"Original photograph"<Icon name="external"/></a><a href=field("license_url") target="_blank" rel="noopener noreferrer">{field("license")}</a></div><small>{field("changes")}</small></article>}
    }).collect_view()}</section><section class="container prose credits-detail"><h2>"More trail photography"</h2><ul>{original.into_iter().map(|asset|view!{<li><a href=asset["source"].as_str().unwrap_or_default().to_string() target="_blank" rel="noopener noreferrer">{format!("{} / {}",asset["name"].as_str().unwrap_or_default(),asset["credit"].as_str().unwrap_or_default())}</a>{format!(". {}. Resized, converted to WebP, and cropped for responsive display.",asset["license"].as_str().unwrap_or_default())}</li>}).collect_view()}</ul><p>"The Meares Glacier and Unakwik Inlet photograph represents the wider Chugach National Forest region. It is not a photograph of Crow Pass or a specific trail segment."</p><h2 id="map-sources">"A map grounded in geography."</h2><p>"Our orientation map combines the BLM historic trail alignment with Natural Earth coastline data. The displayed alignment is the historic main route; it does not represent every connecting trail or the current dog sled race route. It is not a navigation, public-access, or conditions map."</p><ul><li><a href="https://www.arcgis.com/home/item.html?id=37787244e85c450c8247c34eb221954c" target="_blank" rel="noopener noreferrer">"BLM historic trail alignment, hosted by the National Park Service"</a></li><li><a href="https://www.naturalearthdata.com/about/terms-of-use/" target="_blank" rel="noopener noreferrer">"Natural Earth / public-domain geographic data"</a></li><li><a href=url("/maps/map-sources.json")>"Map provenance, place coordinates, and source notes"</a></li></ul><h2>"History, with care."</h2><p>"Editorial sources include the Alliance, the Bureau of Land Management, ARLIS, and the University of Alaska Fairbanks. Historical and cultural copy is prepared for Alliance review. Archival images are labeled with their historical context."</p><ul><li><a href="https://www.iditarod100.org/">"Iditarod Historic Trail Alliance"</a></li><li><a href="https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod">"Bureau of Land Management: Iditarod National Historic Trail"</a></li><li><a href="https://www.arlis.org/collections/iditarod-historic-trail/">"ARLIS: Historic trail collections"</a></li><li><a href="https://jukebox.uaf.edu/">"University of Alaska Fairbanks: Project Jukebox"</a></li></ul><h2>"Type & materials"</h2><p>"Fraunces and DM Sans are distributed under the SIL Open Font License. Font licenses accompany the website. The Nome photograph and its resized WebP derivatives are available under CC BY-SA 2.0."</p></section>}
}
