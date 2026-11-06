//! Editorial presentation for the living, CMS-managed trail.
use crate::{cms::{self, Content, ContentMeta, BodyText}, components::*, motion::{Reveal, SeasonalHero}, paths::url};
use leptos::prelude::*;
use leptos_meta::Title;

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
            <crate::explorer::TrailExplorer/>
            <section class="journal-opening container" id="discover">
                <Reveal><div class="section-marker"><span>"02"</span>"A PLACE IN THE STORY"</div><div class="opening-grid"><h2>"One trail."<br/><em>"An Alaska of"<br/>"possibilities."</em></h2><div class="opening-copy"><p class="lead">"A path through extraordinary landscapes. A connection between people, places, and generations."</p><p>"The Iditarod is a living network of winter trails, community connections, and shared history. There are many ways to find your place in its story."</p><a class="text-link" href=url("/about")>"Meet the Alliance"<Icon/></a></div></div></Reveal>
                {(!home_body.is_empty()).then(||view!{<div class="prose cms-home-intro"><BodyText text=home_body/></div>})}
                <div class="journal-pathways">
                    <a class="pathway-major" href=url("/explore")><img src=url("/images/library-travel/crow-pass-hiker.webp") alt="A hiker follows a green alpine valley on the Crow Pass route near Girdwood" loading="lazy" width="1600" height="1100"/><div class="photo-scrim"/><span class="pathway-number">"01 / EXPERIENCE"</span><div><h3>"Go a little"<br/><em>"further."</em></h3><span class="pathway-link">"Explore places along the trail"<Icon/></span></div></a>
                    <div class="pathway-side"><a class="pathway-small" href=url("/stories")><div><span class="eyebrow">"02 / DISCOVER"</span><h3>"History with"<br/><em>"a heartbeat."</em></h3><span class="text-link">"Stories & heritage"<Icon/></span></div><img src=url("/images/editorial/seward-waterfront-archive-small.webp") alt="Historic Seward waterfront before the 1964 earthquake" loading="lazy" width="800" height="600"/></a><a class="pathway-note" href=url("/get-involved")><span class="eyebrow">"03 / CARRY IT FORWARD"</span><h3>"A lasting trail."<br/><em>"A shared responsibility."</em></h3><p>"Help preserve its history, care for its places, and connect the next generation."</p><span class="text-link">"Make a difference"<Icon/></span><span class="note-mark" aria-hidden="true"><Icon name="mountain"/></span></a></div>
                </div>
            </section>

            <section class="journal-places container section"><Reveal><div class="section-heading"><div><span class="section-marker"><span>"03"</span>"PLACES TO BEGIN"</span><h2>"Find your own"<br/><em>"way into Alaska."</em></h2></div><div class="heading-aside"><p>"Coastal communities. Mountain country. The quiet of a winter trail."</p><a class="text-link" href=url("/explore")>"Explore the trail"<Icon/></a></div></div></Reveal><cms::ManagedHomePlaces/></section>
            <JournalStories/>
            <section class="journal-invitation"><div class="container"><span class="eyebrow">"THE NEXT CHAPTER"</span><div><h2>"Some things are"<br/><em>"worth carrying forward."</em></h2><div><p>"The trail's future is a shared undertaking. Bring your curiosity, your time, or your support."</p><a class="button button-green" href=url("/get-involved")>"Be part of the trail"<Icon/></a></div></div><span class="invitation-rule"/></div></section>
        </div>
    }
}

#[component]
pub fn JournalStories() -> impl IntoView {
    let stories = cms::items("story");
    let preferred = cms::page("home").and_then(|p|p.field("featured_story").parse::<u64>().ok());
    let featured = stories.iter().find(|s|Some(s.id)==preferred).or_else(||stories.iter().find(|s|s.slug=="a-living-connection")).or(stories.first()).cloned();
    let rest: Vec<_> = stories.iter().filter(|s| featured.as_ref().is_none_or(|f|f.id!=s.id)).cloned().collect();
    view! {<section class="journal-stories"><div class="container"><div class="section-heading"><div><span class="section-marker"><span>"04"</span>"THE TRAIL JOURNAL"</span><h2>"Every mile"<br/><em>"carries a story."</em></h2></div><a class="text-link" href=url("/stories")>"All stories"<Icon/></a></div><div class="journal-stories-grid">
        {featured.map(|s|view!{<a class="journal-lead-story" href=url(&s.path)><div class="journal-story-image"><img src=crate::photography::thumbnail(&s) srcset=crate::photography::srcset(&s) sizes="(max-width: 760px) 100vw, 50vw" style=s.image_position() alt=s.field("image_alt") loading="lazy" width="1200" height="800"/><span class="journal-image-label">"THE LIVING TRAIL"</span></div><div><span class="eyebrow">{s.field("category")}</span><h3>{s.title.clone()}</h3><p>{s.summary.clone()}</p><span class="text-link">"Read the story"<Icon/></span></div></a>})}
        <div class="journal-story-list">{rest.into_iter().enumerate().map(|(index,s)|view!{<a class="journal-story-row" href=url(&s.path)><span class="story-order">{format!("0{}",index+2)}</span><div><span class="eyebrow">{s.field("category")}</span><h3>{s.title.clone()}</h3><p>{s.summary.clone()}</p><span class="story-reading">{format!("{} min read",reading_time(&s))}<Icon/></span></div><img src=crate::photography::thumbnail(&s) srcset=crate::photography::srcset(&s) sizes="(max-width: 760px) 100vw, 50vw" style=s.image_position() alt=s.field("image_alt") loading="lazy" width="320" height="380"/></a>}).collect_view()}<a class="journal-archive-link" href=url("/resources")><Icon name="book"/><div><span class="eyebrow">"A LITTLE DEEPER"</span><h3>"Open the archive."</h3><p>"Maps, history, and resources for curious minds."</p></div><Icon/></a></div>
    </div></div></section>}
}

#[component]
pub fn JournalArticle(item: Content) -> impl IntoView {
    if item.kind=="place" {view!{<crate::interiors::DestinationGuide item/>}.into_any()} else {view!{<crate::interiors::MagazineStory item/>}.into_any()}
}

#[component]
pub fn EditorialCredits() -> impl IntoView {
    view!{<Title text="Photography & map sources | The Living Trail"/><section class="container page-intro"><span class="eyebrow">"THE TRAIL, THROUGH MANY EYES"</span><h1>"Every image has a story."</h1><p class="intro-copy">"Communities, journeys, and the work of caring for a trail. Explore the collection and the original records behind every photograph."</p><p class="library-context-note">"Historic photographs keep their dates and places. Every record includes the photographer, original source, and terms for reuse."</p></section><crate::photography::PhotoLibrary/><section class="container prose credits-detail"><p>"The Meares Glacier and Unakwik Inlet photograph represents the wider Chugach National Forest region, beyond the historic trail corridor."</p><h2 id="map-sources">"A map grounded in geography."</h2><p>"Our orientation map combines the BLM historic trail alignment with Natural Earth coastline data. The displayed alignment is the historic main route; it does not represent every connecting trail or the current dog sled race route. It is not a navigation, public-access, or conditions map."</p><ul><li><a href="https://www.arcgis.com/home/item.html?id=37787244e85c450c8247c34eb221954c" target="_blank" rel="noopener noreferrer">"BLM historic trail alignment, hosted by the National Park Service"</a></li><li><a href="https://www.naturalearthdata.com/about/terms-of-use/" target="_blank" rel="noopener noreferrer">"Natural Earth / public-domain geographic data"</a></li><li><a href=url("/maps/explorer-map-sources.json")>"Explorer rivers, graticule and geographic labels"</a></li><li><a href=url("/maps/explorer-places.json")>"Place histories, seasonal guidance and sources"</a></li><li><a href=url("/maps/map-sources.json")>"Map provenance, place coordinates, and source notes"</a></li></ul><h2>"History, with care."</h2><p>"Editorial sources include the Alliance, the Bureau of Land Management, ARLIS, and the University of Alaska Fairbanks. Historical and cultural copy is prepared for Alliance review. Archival images are labeled with their historical context."</p><ul><li><a href="https://www.iditarod100.org/">"Iditarod Historic Trail Alliance"</a></li><li><a href="https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod">"Bureau of Land Management: Iditarod National Historic Trail"</a></li><li><a href="https://www.arlis.org/collections/iditarod-historic-trail/">"ARLIS: Historic trail collections"</a></li><li><a href="https://jukebox.uaf.edu/">"University of Alaska Fairbanks: Project Jukebox"</a></li></ul><h2>"Type & materials"</h2><p>"Fraunces and DM Sans are distributed under the SIL Open Font License. Font licenses accompany the website. Each photograph and its WebP derivatives retain the source license shown in the collection. ShareAlike photographs remain available under their respective ShareAlike licenses."</p></section>}
}
