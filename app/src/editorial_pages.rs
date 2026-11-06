//! Public page compositions and optional service handoffs.
use crate::{cms::{self, ContentCard}, components::*, editorial::JournalStories, paths::url};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn StoriesIndex() -> impl IntoView {
    view! {<cms::ManagedIntro slug="stories"/><div class="journal-index"><JournalStories/></div>
        <section class="container archive-invitation"><img src=url("/images/editorial/seward-waterfront-archive.webp") alt="Seward waterfront before the 1964 earthquake" loading="lazy" width="1799" height="1176"/><div><span class="eyebrow">"LOOK A LITTLE CLOSER"</span><h2>"The past has"<br/><em>"more to tell."</em></h2><p>"Follow a photograph, an oral history, or a map into the trail's wider story."</p><a class="text-link" href=url("/resources")>"Explore the archive"<Icon/></a><small>"Seward waterfront, before 1964 / USGS"</small></div></section>}
}

#[component]
pub fn AlliancePage() -> impl IntoView {
    view!{<crate::interiors::AllianceHistory item=cms::page("about").unwrap_or_default()/>}
}

#[component]
fn ServiceCard(key: &'static str, number: &'static str, title: &'static str, description: &'static str, action: &'static str, waiting: &'static str) -> impl IntoView {
    let destination = cms::public_link(key);
    view! {<article class="participation-card"><span class="participation-number">{number}</span><h3>{title}</h3><p>{description}</p>{match destination {Some(href)=>view!{<a class="button button-green" href=href target="_blank" rel="noopener noreferrer">{action}<Icon name="external"/></a><small>"Continue with the Alliance's service."</small>}.into_any(),None=>view!{<p class="availability-note">{waiting}</p>}.into_any()}}</article>}
}

#[component]
pub fn SupportPage() -> impl IntoView {
    let campaigns=cms::items("campaign");let events=cms::items("event");
    let image=crate::photography::with_fallback(cms::page("get-involved").unwrap_or_default(),"old-woman-mountain");
    view! {<cms::ManagedIntro slug="get-involved" show_photo=false/><section class="support-landscape"><img src=image.image() srcset=crate::photography::srcset(&image) sizes="100vw" style=image.image_position() alt=image.field("image_alt") width="1600" height="1000" fetchpriority="high"/><div class="support-landscape-shade"/><div class="container"><span class="eyebrow">"A CONNECTION WORTH KEEPING"</span><h2>"For the trail."<br/><em>"For those to come."</em></h2><p>"Help keep a living part of Alaska's heritage within reach of future generations."</p></div></section><div class="container support-photo-caption"><crate::photography::PhotoAttribution item=image/></div>
        <section class="container participation"><div class="section-heading"><div><span class="section-marker"><span>"01"</span>"FIND YOUR PART"</span><h2>"Small beginnings."<br/><em>"Lasting connections."</em></h2></div><p>"A gift, a membership, a little of your time. Each begins with a shared care for this extraordinary trail."</p></div><div class="participation-grid">
            <ServiceCard key="donation_url" number="01 / GIVE" title="Keep the story going." description="Support the work of preserving and sharing the historic trail." action="Make a gift" waiting="Online giving will be available here when it opens."/>
            <ServiceCard key="membership_url" number="02 / BELONG" title="Become part of it." description="Connect with others who care about the trail and its future." action="Become a member" waiting="Membership details will be shared here when available."/>
            <div id="volunteer"><ServiceCard key="volunteer_url" number="03 / TAKE PART" title="Bring what you can." description="Share your time, knowledge, or enthusiasm for the trail." action="Find an opportunity" waiting="Volunteer opportunities will be shared here as they become available."/></div>
        </div><a class="text-link participation-about" href=url("/about")>"Discover the Alliance's work"<Icon/></a></section>
        {(!campaigns.is_empty()).then(||view!{<section class="container participation-list"><span class="eyebrow">"CARRY IT FORWARD"</span><h2>"Work worth supporting."</h2><div class="three-grid">{campaigns.into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div></section>})}
        {(!events.is_empty()).then(||view!{<section class="container participation-list"><span class="eyebrow">"COME TOGETHER"</span><h2>"Along the way."</h2><div class="three-grid">{events.into_iter().map(|item|view!{<ContentCard item/>}).collect_view()}</div></section>})}}
}

#[component]
pub fn TrailShop() -> impl IntoView {
    let destination=cms::public_link("store_url");
    let image=crate::photography::with_fallback(cms::page("store").unwrap_or_default(),"seward-trail-monument");
    view! {<cms::ManagedIntro slug="store" show_photo=false/><section class="container shop-opening"><figure><img src=image.image() srcset=crate::photography::srcset(&image) sizes="(max-width: 760px) 100vw, 50vw" style=image.image_position() alt=image.field("image_alt") width="1600" height="1200" fetchpriority="high"/><figcaption><crate::photography::PhotoAttribution item=image/></figcaption></figure><div><span class="eyebrow">"A LITTLE CLOSER TO THE TRAIL"</span><h2>"A memory."<br/>"A story."<br/><em>"A connection."</em></h2><p>"Keep the places and stories of the historic trail close, wherever your own journey takes you."</p>{match destination {Some(href)=>view!{<a class="button button-green" href=href target="_blank" rel="noopener noreferrer">"Explore the trail shop"<Icon name="external"/></a><p class="shop-status">"Browse the collection and complete your purchase in the Alliance's online store."</p>}.into_any(),None=>view!{<div class="shop-coming"><span class="eyebrow">"SOMETHING TO LOOK FORWARD TO"</span><h3>"The next collection starts here."</h3><p>"The trail shop will open here when the collection is ready."</p></div>}.into_any()}}</div></section><section class="shop-discover"><div class="container"><div><span class="eyebrow">"IN THE MEANTIME"</span><h2>"Take a story with you."</h2></div><a class="button button-green" href=url("/stories")>"Open the trail journal"<Icon/></a></div></section>}
}

#[component]
pub fn ResourcePrelude() -> impl IntoView {
    let image=crate::photography::with_fallback(cms::page("resources").unwrap_or_default(),"mcgrath-post-office-1914");
    view! {<section class="container resource-prelude"><div><span class="eyebrow">"YOUR FIELD LIBRARY"</span><h2>"A little preparation."<br/><em>"A world to discover."</em></h2><p>"Find a starting point for a visit, a classroom conversation, or a deeper look into the trail's past."</p><a class="text-link" href="#resource-query">"Browse the collection"<Icon name="down"/></a></div><figure class="resource-prelude-photo"><img src=image.image() srcset=crate::photography::srcset(&image) sizes="(max-width: 760px) 100vw, 50vw" style=image.image_position() alt=image.field("image_alt") width="1200" height="800" loading="lazy"/><figcaption><crate::photography::PhotoAttribution item=image/></figcaption></figure></section>}
}

#[component]
pub fn ReviewGuide() -> impl IntoView {
    view! {<Title text="Explore the website | The Living Trail"/><PageIntro kicker="A NEW CHAPTER FOR THE TRAIL" title="Take the scenic route." description="Explore the website, then open Trail Studio to see how the team can keep it current."/><section class="container review-guide"><div class="tour-steps">{[("01","Explore the landscape","Move between trail chapters, filter places, and follow the stories.","/explore"),("02","Discover the collection","Search the resource library and open original documents.","/resources"),("03","See the publishing workflow","Sign in with your review account to save a draft, preview a page, and publish an update.","/admin")].into_iter().map(|(n,t,d,h)|view!{<a class="tour-step" href=url(h)><span>{n}</span><div><h2>{t}</h2><p>{d}</p></div><Icon/></a>}).collect_view()}</div><div class="review-note"><h2>"Ready for your review."</h2><p>"This is the proposed website for the Iditarod Historic Trail Alliance. Editorial content and identity treatments await approval. The existing official site remains at iditarod100.org."</p><p>"Store, giving, membership, and newsletter destinations can be added in Trail Studio when approved. They are not activated in this review environment."</p><a class="text-link" href="https://www.iditarod100.org/" target="_blank" rel="noopener noreferrer">"Visit the current Alliance website"<Icon name="external"/></a></div></section>}
}
