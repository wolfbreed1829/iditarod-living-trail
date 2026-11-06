use crate::motion::{Reveal, SeasonalHero, TrailJourney};
use crate::paths::url;
use crate::{components::*, data::*};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
fn PlaceCard(place: Place) -> impl IntoView {
    view! {<a class="place-card" href=url(&format!("/places/{}",place.slug))><div class="card-image"><img src=url(&format!("/images/{}-small.webp",place.image)) alt=place.alt loading="lazy" width="800" height="540"/><span class="image-tag">{place.season}</span><span class="circle-arrow"><Icon/></span></div><div class="card-copy"><span class="eyebrow">{place.region}</span><h3>{place.name}</h3><p>{place.intro}</p><span class="card-meta"><Icon name="pin"/>{place.activity}</span></div></a>}
}

#[component]
fn StoryCard(story: Story) -> impl IntoView {
    view! {<a class="story-card" href=url(&format!("/stories/{}",story.slug))><div class="card-image"><img src=url(&format!("/images/{}-small.webp",story.image)) alt=story.alt loading="lazy" width="800" height="540"/><span class="circle-arrow"><Icon/></span></div><p class="eyebrow">{story.category}</p><h3>{story.title}</h3><p>{story.intro}</p><span class="text-link">"Read the story"<Icon/></span></a>}
}

#[component]
pub fn Home() -> impl IntoView {
    let paused = RwSignal::new(false);
    view! {
        <Title text="The Living Trail | Iditarod Historic Trail Alliance"/>
        <div class="motion-scene" class:motion-paused=move || paused.get()>
        <SeasonalHero paused/>
        {crate::cms::page("home").map(|item|view!{<crate::cms::ContentMeta item=item.clone()/>
            <section class="container prose cms-home-intro"><crate::cms::BodyText text=item.body/></section>})}
        <div class="trail-facts"><div class="container"><p><strong>"2,400"</strong><span>"MILES OF CONNECTED TRAILS"</span></p><span class="fact-divider"/><p><strong>"1978"</strong><span>"A NATIONAL HISTORIC TRAIL"</span></p><span class="fact-divider"/><p class="fact-words"><Icon name="pin"/><span>"From the shores of Seward"<br/><strong>"to the heart of Nome."</strong></span></p></div></div>
        <section class="section container" id="discover"><Reveal><div class="section-heading"><div><Eyebrow>"YOUR CONNECTION STARTS HERE"</Eyebrow><h2>"One trail."<br/><em>"Countless ways to belong."</em></h2></div><p>"A walk by the water. A story passed down. A journey into winter. Find your own connection to a trail that has brought people together for generations."</p></div></Reveal>
                <Reveal>
            <div class="experience-grid">
                <a href=url("/explore") class="experience-card"><img src=url("/images/summer-small.webp") alt="Glacier and mountains in Chugach National Forest" loading="lazy" width="800" height="600"/><div><span class="eyebrow">"01 / EXPERIENCE"</span><h3>"Find your next"<br/>"adventure."</h3><span>"Explore the trail"<Icon/></span></div></a>
                <a href=url("/stories") class="experience-card"><img src=url("/images/dogs-small.webp") alt="Sled dogs resting on the Iditarod Trail" loading="lazy" width="800" height="600"/><div><span class="eyebrow">"02 / DISCOVER"</span><h3>"Every mile"<br/>"has a story."</h3><span>"Meet the living trail"<Icon/></span></div></a>
                <a href=url("/get-involved") class="experience-card"><img src=url("/images/cycling-small.webp") alt="Winter cyclists traveling the trail together" loading="lazy" width="800" height="600"/><div><span class="eyebrow">"03 / PROTECT"</span><h3>"Be part of"<br/>"what comes next."</h3><span>"Make a difference"<Icon/></span></div></a>
            </div></Reveal>
        </section>
        <TrailJourney/>
        <section class="section container"><Reveal><div class="section-heading align-end"><div><Eyebrow>"GO A LITTLE FURTHER"</Eyebrow><h2>"Your trail starts here."</h2></div><a class="text-link" href=url("/explore")>"Explore places & experiences"<Icon/></a></div></Reveal><Reveal>{if crate::cms::catalog().is_some(){view!{<crate::cms::ManagedHomePlaces/>}.into_any()}else{view!{<div class="three-grid">{PLACES.into_iter().map(|place|view!{<PlaceCard place/>}).collect_view()}</div>}.into_any()}}</Reveal></section>
        {if crate::cms::catalog().is_some(){view!{<crate::cms::ManagedFeature/>}.into_any()}else{view!{<section class="feature-story"><img src=url("/images/dogs.webp") alt="A sled dog team resting together in the snow" loading="lazy" width="2000" height="900"/><div class="feature-story-shade"/><div class="container"><div><Eyebrow>"STORIES FROM THE TRAIL"</Eyebrow><h2>"Some connections"<br/>"never grow old."</h2><p>"Meet a legacy of winter travel, shared knowledge, and the enduring bond between people and place."</p><a href=url("/stories/the-winter-way") class="button button-light">"Read the winter story"<Icon/></a></div></div></section>}.into_any()}}
        <Reveal><section class="support-callout container"><span class="compass" aria-hidden="true">"✳"</span><Eyebrow>"KEEP THE CONNECTION ALIVE"</Eyebrow><h2>"The next chapter"<br/>"includes you."</h2><p>"Help protect the trail, share its stories, and inspire the next generation of stewards."</p><a href=url("/get-involved") class="button button-green">"Become part of the trail"<Icon/></a></section></Reveal>
        </div>
    }
}

#[component]
pub fn Explore() -> impl IntoView {
    let season = RwSignal::new("All".to_string());
    let region = RwSignal::new("All regions".to_string());
    let query = RwSignal::new(String::new());
    let filtered = Memo::new(move |_| {
        PLACES
            .into_iter()
            .filter(|p| {
                (season.get() == "All" || p.season == "All seasons" || p.season == season.get())
                    && (region.get() == "All regions" || p.region == region.get())
                    && format!("{} {} {}", p.name, p.region, p.activity)
                        .to_lowercase()
                        .contains(&query.get().to_lowercase())
            })
            .collect::<Vec<_>>()
    });
    view! {<Title text="Find your trail | Iditarod"/><PageIntro kicker="PLAN A TRIP" title="Find your kind of adventure." description="From a coastal afternoon to a winter journey, discover a connection that feels like yours."/>
        <div class="container"><div class="explore-banner"><img src=url("/images/winter.webp") alt="Dog team and musher traveling a winter trail near a shelter cabin" width="1200" height="676"/><div><span class="eyebrow">"A TRAIL FOR ALL SEASONS"</span><h2>"Start with curiosity."<br/>"Travel with care."</h2></div></div>
        <div class="filter-bar"><div class="filter-pills" role="group" aria-label="Filter by season">{["All","Summer","Winter"].into_iter().map(|s|view!{<button class:active=move||season.get()==s aria-pressed=move||(season.get()==s).to_string() on:click=move |_|season.set(s.to_string())>{if s=="All"{"All seasons"}else{s}}</button>}).collect_view()}</div><label class="region-select"><span class="sr-only">"Region"</span><select prop:value=move||region.get() on:change=move|e|region.set(event_target_value(&e))><option>"All regions"</option><option>"Southern trail"</option><option>"Southcentral Alaska"</option><option>"Across the trail"</option></select></label><label class="search-field"><Icon name="search"/><span class="sr-only">"Search places"</span><input type="search" placeholder="Find a place or activity" prop:value=move||query.get() on:input=move|e|query.set(event_target_value(&e))/></label></div>
        <p class="results-count" aria-live="polite">{move||format!("{} places to begin",filtered.get().len())}</p><div class="three-grid places-results">{move||filtered.get().into_iter().map(|place|view!{<PlaceCard place/>}).collect_view()}</div>
        <Show when=move||filtered.get().is_empty()><div class="empty-state"><h2>"A different path, perhaps?"</h2><p>"Try another season or region to discover more places."</p><button class="button button-green" on:click=move |_|{season.set("All".into());region.set("All regions".into());query.set(String::new());}>"Clear filters"<Icon/></button></div></Show>
        <div class="resource-callout"><Icon name="book"/><div><h3>"A little preparation goes a long way."</h3><p>"Check current conditions, local access, and land-manager guidance before you travel. This preview is a starting point for discovery."</p></div><a class="text-link" href=url("/resources")>"Visitor resources"<Icon/></a></div></div>
    }
}

#[component]
pub fn PlaceDetail() -> impl IntoView {
    let params = use_params_map();
    view! {{move||{
        let slug=params.get().get("slug").unwrap_or_default();
        match PLACES.into_iter().find(|p|p.slug==slug){None=>view!{<NotFound/>}.into_any(),Some(p)=>view!{
            <Title text=format!("{} | Iditarod",p.name)/><div class="container breadcrumb"><a href=url("/explore")>"Plan a trip"</a><span>"/"</span><span>{p.name}</span></div>
            <section class="detail-hero"><img src=url(&format!("/images/{}.webp",p.image)) alt=p.alt fetchpriority="high"/><div class="container"><Eyebrow>{p.region}</Eyebrow><h1>{p.name}</h1><p>{p.intro}</p></div></section>
            <p class="photo-caption container">{p.caption}</p>
            <div class="container article-grid section"><article class="prose"><Eyebrow>"MAKE A CONNECTION"</Eyebrow><h2>"A place to begin."</h2><p class="lead">{p.story}</p><h3>"Before you go"</h3><p>"Plan for the season, respect local communities and private lands, and use current guidance from the relevant land manager. Access and conditions can change."</p><a class="text-link" href=url(p.resource) target="_blank" rel="noopener noreferrer">"Open official visitor information"<Icon name="external"/></a><h3>"Leave room for discovery."</h3><p>"A visit is also a chance to listen. Explore local history, seek out community perspectives, and take time to understand the place beyond the view."</p><a href=url("/stories/a-living-connection") class="text-link">"Discover the trail's story"<Icon/></a></article>
            <aside class="visit-card"><span class="eyebrow">"AT A GLANCE"</span><h3>"Your visit"</h3><dl><div><dt>"Region"</dt><dd>{p.region}</dd></div><div><dt>"Explore in"</dt><dd>{p.season}</dd></div><div><dt>"Find your connection"</dt><dd>{p.activity}</dd></div></dl><a class="button button-green" href=url(p.resource) target="_blank" rel="noopener noreferrer">"Visitor information"<Icon name="external"/></a><a class="text-link" href=url("/resources")>"Maps & guides"<Icon/></a><p class="form-note">"Confirm current conditions with local sources. This is a design preview, not a live trail report."</p></aside></div>
            <section class="container section related-section"><Eyebrow>"KEEP EXPLORING"</Eyebrow><h2>"Another way to connect."</h2><div class="two-grid">{PLACES.into_iter().filter(|other|other.slug!=p.slug).map(|place|view!{<PlaceCard place/>}).collect_view()}</div></section>
        }.into_any()}
    }}}
}

#[component]
pub fn Stories() -> impl IntoView {
    view! {<Title text="Stories of the living trail | Iditarod"/><PageIntro kicker="LEARN & DISCOVER" title="Every mile has a story." description="Discover the people, traditions, and landscapes that connect the trail's past to its living present."/>
        <section class="container stories-feature"><img src=url("/images/dawn.webp") alt="Soft pink winter light on mountains along the Iditarod Trail"/><div><Eyebrow>"THE LIVING TRAIL"</Eyebrow><h2>"A connection"<br/>"across generations."</h2><p>"Follow a story that began long before the gold rush, and continues in the communities of today."</p><a href=url("/stories/a-living-connection") class="button button-green">"Begin the story"<Icon/></a></div></section>
        <section class="container section"><div class="section-heading"><div><Eyebrow>"LOOK CLOSER"</Eyebrow><h2>"Stories worth following."</h2></div></div><div class="three-grid">{STORIES.into_iter().map(|story|view!{<StoryCard story/>}).collect_view()}</div></section>
        <section class="learning-banner"><div class="container"><Icon name="book"/><div><Eyebrow>"THE TRAIL IS A CLASSROOM"</Eyebrow><h2>"Big discoveries."<br/>"Curious minds."</h2><p>"Explore activities, education resources, and ways to bring the trail into every classroom."</p><a class="button button-light" href=url("/resources")>"For educators & families"<Icon/></a></div></div></section>
    }
}

#[component]
pub fn StoryDetail() -> impl IntoView {
    let params = use_params_map();
    let chapter = RwSignal::new(0usize);
    view! {{move||{
        let slug=params.get().get("slug").unwrap_or_default();
        match STORIES.into_iter().find(|s|s.slug==slug){None=>view!{<NotFound/>}.into_any(),Some(s)=>view!{
            <Title text=format!("{} | Iditarod",s.title)/><div class="container breadcrumb"><a href=url("/stories")>"Stories & history"</a><span>"/"</span><span>{s.category}</span></div>
            <header class="article-header container"><Eyebrow>{s.category}</Eyebrow><h1>{s.title}</h1><p>{s.intro}</p><span class="article-byline">"THE LIVING TRAIL · EDITORIAL DESIGN PREVIEW"</span></header>
            <figure class="article-photo"><img src=url(&format!("/images/{}.webp",s.image)) alt=s.alt/><figcaption><a href=url("/credits")>"Contemporary trail and regional photography · Full credits"</a></figcaption></figure>
            <div class="container article-grid section"><article class="prose"><p class="dropcap">{s.paragraphs[0]}</p><h2>"A story still unfolding."</h2><p>{s.paragraphs[1]}</p><blockquote>"A trail is kept alive by the people who know it, use it, and care for it."<cite>"— THE LIVING TRAIL · CONCEPT EDITORIAL"</cite></blockquote><p>{s.paragraphs[2]}</p><a href=url(s.source) class="text-link" target="_blank" rel="noopener noreferrer">"Continue with the original resources"<Icon name="external"/></a></article><aside class="story-aside"><TrailDrawing/><p>"People. Place. Connection."</p><a href=url("/get-involved") class="text-link">"Help carry the story forward"<Icon/></a></aside></div>
            <section class="timeline-section"><div class="container"><Eyebrow>"THE TRAIL THROUGH TIME"</Eyebrow><h2>"Many chapters. One living trail."</h2><div class="timeline-tabs" role="group" aria-label="Choose a chapter of trail history">{["Deep roots","Gold-rush journeys","1978","Today"].into_iter().enumerate().map(|(i,label)|view!{<button class:active=move||chapter.get()==i aria-pressed=move||(chapter.get()==i).to_string() on:click=move |_|chapter.set(i)><span>{format!("0{}",i+1)}</span>{label}</button>}).collect_view()}</div><div class="timeline-content" aria-live="polite"><h3>{move||["Connection comes first.","A route takes shape.","A national recognition.","The next chapter is ours."][chapter.get()]}</h3><p>{move||["Alaska Native travel networks connect communities across the landscape, long before the gold-rush route.","Winter routes carry people, mail, and supplies between communities, coastal ports, and mining districts.","Congress designates the Iditarod National Historic Trail, recognizing a remarkable network of historic winter routes.","Communities, land managers, volunteers, and the Alliance continue the work of caring for the trail and sharing its stories."][chapter.get()]}</p></div></div></section>
            <section class="container section"><div class="section-heading"><h2>"Keep following the story."</h2><a href=url("/stories") class="text-link">"All stories"<Icon/></a></div><div class="two-grid">{STORIES.into_iter().filter(|other|other.slug!=s.slug).map(|story|view!{<StoryCard story/>}).collect_view()}</div></section>
        }.into_any()}
    }}}
}

#[component]
pub fn Support() -> impl IntoView { view!{<crate::support::SupportPage/>} }

#[component]
pub fn Store() -> impl IntoView {
    view!{<crate::editorial_pages::TrailShop/>}
}

#[component]
pub fn About() -> impl IntoView {
    view! {<Title text="Our alliance | Iditarod"/><PageIntro kicker="THE IDITAROD HISTORIC TRAIL ALLIANCE" title="Connected by a trail. United by its future." description="The Alliance brings people together to promote, preserve, and protect the Iditarod National Historic Trail."/>
        <section class="container about-image"><img src=url("/images/winter.webp") alt="A musher and dog team traveling the Iditarod Trail"/></section><section class="container section two-grid"><div><Eyebrow>"A SHARED RESPONSIBILITY"</Eyebrow><h2>"The trail's future"<br/>"is a collective story."</h2></div><div class="prose"><p class="lead">"The Iditarod Historic Trail Alliance is a statewide nonprofit organization founded in 1999. Its work connects education, history, recreation, and stewardship."</p><p>"Through partnerships with communities, landowners, agencies, and volunteers, the Alliance helps keep the trail's heritage present and its future possible."</p><p>"This design preview brings that work into focus. Organizational details and final editorial content would be developed with the Alliance."</p><a href="https://www.iditarod100.org/about-us.html" target="_blank" rel="noopener noreferrer" class="text-link">"Meet the Alliance on its current website"<Icon name="external"/></a></div></section>
        <section class="mission-band"><div class="container three-grid">{[("01","Preserve","Help care for the trail and its historic character."),("02","Connect","Bring people closer to the trail's stories and communities."),("03","Inspire","Encourage learning, discovery, and stewardship.")].into_iter().map(|(n,t,d)|view!{<div><span class="eyebrow">{n}</span><h2>{t}</h2><p>{d}</p></div>}).collect_view()}</div></section><section class="support-callout container"><Eyebrow>"BECOME PART OF THE STORY"</Eyebrow><h2>"A lasting trail"<br/>"starts with all of us."</h2><a href=url("/get-involved") class="button button-green">"Get involved"<Icon/></a></section>
    }
}

#[component]
pub fn Resources() -> impl IntoView {
    let query = RwSignal::new(String::new());
    let category = RwSignal::new("All resources");
    let resources = [
        (
            "Visitor guide",
            "Explore",
            "A guide to the historic trail, its communities, and ways to experience it.",
            "https://www.iditarod100.org/uploads/5/2/4/5/52459009/inht-vg.pdf",
            "PDF",
        ),
        (
            "Maps & trail information",
            "Explore",
            "Official BLM information to begin your research.",
            "https://www.blm.gov/visit/iditarod-trail",
            "WEBSITE",
        ),
        (
            "Iditarod Trail to Every Classroom",
            "Learn",
            "Discover the Alliance's place-based education program.",
            "https://www.iditarod100.org/itrec-education-program.html",
            "WEBSITE",
        ),
        (
            "Junior Trailblazer",
            "Learn",
            "A trail activity booklet for curious young explorers.",
            "https://www.iditarod100.org/uploads/5/2/4/5/52459009/iditarod_jrweb.pdf",
            "PDF",
        ),
        (
            "Project Jukebox",
            "History",
            "Explore oral histories through the University of Alaska Fairbanks.",
            "https://jukebox.uaf.edu/",
            "WEBSITE",
        ),
        (
            "Historic trail collections",
            "History",
            "Discover documents, photographs, and research in the ARLIS collection.",
            "https://www.arlis.org/collections/iditarod-historic-trail/",
            "ARCHIVE",
        ),
        (
            "Passport stamp program",
            "Explore",
            "Find the Alliance's information about trail passport locations.",
            "https://www.iditarod100.org/passport-stamp-program.html",
            "WEBSITE",
        ),
    ];
    let filtered = Memo::new(move |_| {
        resources
            .into_iter()
            .filter(|(name, cat, desc, _, _)| {
                (category.get() == "All resources" || *cat == category.get())
                    && format!("{name} {desc}")
                        .to_lowercase()
                        .contains(&query.get().to_lowercase())
            })
            .collect::<Vec<_>>()
    });
    view! {<Title text="Resources for curious minds | Iditarod"/><PageIntro kicker="EXPLORE. LEARN. GO FURTHER." title="A good place to start." description="Find visitor information, classroom resources, and stories from the trail's archives."/><section class="container resource-library"><div class="resource-search"><Icon name="search"/><label class="sr-only" for="resource-query">"Search resources"</label><input id="resource-query" type="search" placeholder="Search guides, history, classroom resources…" prop:value=move||query.get() on:input=move|e|query.set(event_target_value(&e))/></div><div class="filter-pills resource-filters" role="group" aria-label="Resource categories">{["All resources","Explore","Learn","History"].into_iter().map(|c|view!{<button class:active=move||category.get()==c aria-pressed=move||(category.get()==c).to_string() on:click=move |_|category.set(c)>{c}</button>}).collect_view()}</div><p class="results-count" aria-live="polite">{move||format!("{} resources",filtered.get().len())}</p><div class="resource-list">{move||filtered.get().into_iter().map(|(name,cat,desc,href,format)|view!{<a href=url(href) class="resource-row" target="_blank" rel="noopener noreferrer"><span class="resource-icon"><Icon name="book"/></span><div><span class="eyebrow">{cat}</span><h3>{name}</h3><p>{desc}</p></div><span class="file-type">{format}</span><Icon name="external"/></a>}).collect_view()}</div><Show when=move||filtered.get().is_empty()><div class="empty-state"><h2>"Let's try another trail."</h2><p>"No resources match that search."</p><button class="button button-green" on:click=move |_|{query.set(String::new());category.set("All resources");}>"Clear search"<Icon/></button></div></Show><p class="form-note">"Resources open on their original websites. Availability and content are maintained by their respective organizations."</p></section>}
}

#[component]
pub fn Preview() -> impl IntoView { view!{<crate::editorial_pages::ReviewGuide/>} }

#[component]
pub fn Credits() -> impl IntoView {
    view! {<Title text="Photography & sources | The Living Trail"/><PageIntro kicker="THE PEOPLE BEHIND THE PICTURES" title="A landscape worth crediting." description="Authentic photographs and original resources help ground this design concept in the real trail."/><section class="container prose credits-content"><h2>"Photography"</h2><p>"Photographs are resized, converted to WebP, and cropped to suit responsive layouts. No photograph is presented as an exact before-and-after comparison."</p><ul><li><a href="https://www.flickr.com/photos/mypubliclands/36599186731/" target="_blank" rel="noopener noreferrer">"Winter landscape — Bob Wick / Bureau of Land Management"</a>". "<a href="https://creativecommons.org/licenses/by/2.0/">"CC BY 2.0"</a>"."</li><li><a href="https://www.flickr.com/photos/mypubliclands/36738421625/" target="_blank" rel="noopener noreferrer">"Sled dogs — Bob Wick / Bureau of Land Management"</a>". "<a href="https://creativecommons.org/licenses/by/2.0/">"CC BY 2.0"</a>"."</li><li><a href="https://www.flickr.com/photos/mypubliclands/36568558672/" target="_blank" rel="noopener noreferrer">"Winter cycling — Bob Wick / Bureau of Land Management"</a>". "<a href="https://creativecommons.org/licenses/by/2.0/">"CC BY 2.0"</a>"."</li><li><a href="https://www.flickr.com/photos/alaska_region/29533359703/" target="_blank" rel="noopener noreferrer">"Meares Glacier and Unakwik Inlet, Chugach National Forest — USDA Forest Service Alaska Region"</a>". Regional imagery, not a photograph of Crow Pass. "<a href="https://creativecommons.org/licenses/by/2.0/">"CC BY 2.0"</a>"."</li><li><a href="https://www.nps.gov/kefj/blogs/one-fjord-to-another.htm" target="_blank" rel="noopener noreferrer">"Resurrection Bay from Seward — NPS / Max Odland"</a>". Public domain."</li><li><a href="https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod" target="_blank" rel="noopener noreferrer">"Musher and dog team near Tripod Flats Public Shelter Cabin — Bureau of Land Management"</a>". Government source image."</li></ul><h2>"Editorial sources"</h2><p>"Concept copy is informed by the Alliance's existing website, the Bureau of Land Management's trail overview, and linked education and visitor resources. Historical and cultural editorial content would be reviewed with the Alliance and relevant communities before production publication."</p><ul><li><a href="https://www.iditarod100.org/">"Iditarod Historic Trail Alliance"</a></li><li><a href="https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod">"BLM: Iditarod National Historic Trail"</a></li><li><a href="https://www.arlis.org/collections/iditarod-historic-trail/">"ARLIS: Historic trail collections"</a></li><li><a href="https://jukebox.uaf.edu/">"University of Alaska Fairbanks: Project Jukebox"</a></li></ul><h2>"Design & type"</h2><p>"The trail emblem, product concepts, and route illustration are original design elements for this proposal. The route illustration is not a geographic or navigation map. DM Sans and Fraunces are distributed under the SIL Open Font License; license files accompany this preview."</p><a href=url("/preview") class="text-link">"About this design preview"<Icon/></a></section>}
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {<Title text="A different trail | Iditarod"/><section class="container empty-state not-found"><Eyebrow>"A SMALL DETOUR"</Eyebrow><h1>"This trail ends here."</h1><p>"The page you're looking for could not be found. There is plenty more to discover."</p><a class="button button-green" href=url("/")>"Back to the beginning"<Icon/></a></section>}
}
