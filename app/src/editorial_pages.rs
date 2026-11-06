//! Public page compositions and optional service handoffs.
use crate::{cms, components::*, editorial::JournalStories, paths::url};
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
pub fn SupportPage() -> impl IntoView { view!{<crate::support::SupportPage/>} }

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
    view!{<Title text="Explore the website | The Living Trail"/><PageIntro kicker="FOLLOW YOUR CURIOSITY" title="Take the scenic route." description="Explore the trail, discover the work, and find your way to help."/><section class="container review-guide"><div class="tour-steps">{[("01","Explore the landscape","Move between real places, seasonal experiences, and destination guides.","/explore"),("02","Discover the collection","Read stories, browse original documents, and follow the historical record.","/resources"),("03","Make a connection","See documented projects and save a support enquiry or offer.","/get-involved"),("04","Care for the website","Staff can manage content and review support requests in Trail Studio.","/admin")].into_iter().map(|(n,t,d,h)|view!{<a class="tour-step" href=url(h)><span>{n}</span><div><h2>{t}</h2><p>{d}</p></div><Icon/></a>}).collect_view()}</div></section>}
}
