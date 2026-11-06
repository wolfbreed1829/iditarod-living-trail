use crate::paths::url;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[derive(Clone, Copy)]
pub struct Cart(pub RwSignal<[u32; 3]>);

#[component]
pub fn Icon(#[prop(default = "arrow")] name: &'static str) -> impl IntoView {
    let path = match name {
        "search" => "M21 21l-5-5M18 10a8 8 0 1 1-16 0 8 8 0 0 1 16 0",
        "pin" => "M20 10c0 6-8 12-8 12S4 16 4 10a8 8 0 1 1 16 0ZM15 10a3 3 0 1 1-6 0 3 3 0 0 1 6 0",
        "bag" => "M5 7h14l1 15H4L5 7ZM8 8V5a4 4 0 0 1 8 0v3",
        "menu" => "M3 6h18M3 12h18M3 18h18",
        "close" => "M5 5l14 14M5 19 19 5",
        "pause" => "M8 5v14M16 5v14",
        "play" => "m8 4 12 8-12 8V4Z",
        "check" => "m4 12 5 5L20 6",
        "sun" => "M12 1v2M12 21v2M1 12h2M21 12h2M4 4l2 2M18 18l2 2M4 20l2-2M18 6l2-2M17 12a5 5 0 1 1-10 0 5 5 0 0 1 10 0",
        "snow" => "M12 2v20M3.3 7l17.4 10M3.3 17 20.7 7M9 4l3 3 3-3M9 20l3-3 3 3",
        "book" => "M12 5C8 2 4 3 2 4v16c3-2 6-2 10 0 4-2 7-2 10 0V4c-3-1-6-2-10 1Zm0 0v15",
        "heart" => "M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1.1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8Z",
        "down" => "M12 3v18M5 14l7 7 7-7",
        "external" => "M14 3h7v7M21 3 10 14M10 3H3v18h18v-7",
        _ => "M4 12h16M13 5l7 7-7 7",
    };
    view! {<svg class="icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d=path/></svg>}
}

#[component]
pub fn Brand() -> impl IntoView {
    view! {<a class="brand" href=url("/") aria-label="Iditarod Historic Trail Alliance home">
        <svg class="brand-mark" viewBox="0 0 64 64" fill="none" aria-hidden="true"><circle cx="32" cy="32" r="29" stroke="currentColor" stroke-width="1"/><path d="M10 39 25 17 35 31 42 22 55 40M22 40l9-13 9 13M30 45l-6 5 9 5" stroke="currentColor" stroke-width="1.5"/><path d="m24 18 2 9 5-3M41 23l2 9 5-2" stroke="currentColor" stroke-width="1"/></svg>
        <span><strong>"IDITAROD"</strong>" "<span>"HISTORIC TRAIL ALLIANCE"</span></span>
    </a>}
}

#[component]
pub fn Header() -> impl IntoView {
    let open = RwSignal::new(false);
    let location = use_location();
    Effect::new(move |_| {
        location.pathname.get();
        open.set(false);
    });
    view! {
        <div class="trail-masthead" role="region" aria-label="Trail information"><span>"ALASKA'S IDITAROD NATIONAL HISTORIC TRAIL"</span><a href=url("/about")>"A living legacy, since 1978"<Icon name="external"/></a></div>
        <header class="site-header" on:keydown=move |ev| {if ev.key() == "Escape" {open.set(false)}}>
            <div class="header-inner"><Brand/>
                <nav id="main-nav" class:open=move || open.get() aria-label="Main navigation">
                    {[ ("/explore", "Plan a trip"), ("/stories", "Learn & discover"), ("/get-involved", "Get involved"), ("/about", "Our alliance") ].into_iter().map(|(href,label)| view!{
                        <a href=url(href) aria-current=move || if location.pathname.get().starts_with(&url(href)) {Some("page")} else {None}>{label}</a>
                    }).collect_view()}
                    <a href=url("/store") class="mobile-store">"Trail shop"</a>
                </nav>
                <div class="header-actions">
                    <a class="icon-button search-link" href=url("/resources") aria-label="Search resources"><Icon name="search"/></a>
                    <a class="icon-button bag-link" href=url("/store") aria-label="Trail shop"><Icon name="bag"/></a>
                    <a class="button button-green header-support" href=url("/get-involved")>"Support the trail"<Icon/></a>
                    <button class="icon-button menu-toggle" aria-label=move || if open.get(){"Close menu"}else{"Open menu"} aria-expanded=move || open.get().to_string() aria-controls="main-nav" on:click=move |_| open.update(|v|*v=!*v)>{move || if open.get(){view!{<Icon name="close"/>}}else{view!{<Icon name="menu"/>}}}</button>
                </div>
            </div>
        </header>
    }
}

#[component]
pub fn Eyebrow(children: Children) -> impl IntoView {
    view! {<p class="eyebrow"><span class="small-line"/>{children()}</p>}
}

#[component]
pub fn PageIntro(
    kicker: &'static str,
    title: &'static str,
    description: &'static str,
) -> impl IntoView {
    let location = use_location();
    if let Some(catalog) = crate::cms::catalog() {
        let slug = location
            .pathname
            .get_untracked()
            .trim_matches('/')
            .to_string();
        let content = if catalog.preview_id.is_some() {
            catalog
                .items
                .iter()
                .find(|i| Some(i.id) == catalog.preview_id && i.kind == "page")
        } else {
            catalog
                .items
                .iter()
                .find(|i| i.kind == "page" && i.slug == slug)
        };
        if let Some(item) = content.cloned() {
            return view!{<crate::cms::ContentMeta item=item.clone()/><section class="page-intro container"><p class="eyebrow"><span class="small-line"/>{item.field("category")}</p><h1>{item.title.clone()}</h1><p class="intro-copy">{item.summary.clone()}</p></section>
                <section class="container prose"><crate::cms::BodyText text=item.body.clone()/></section>
                {(!item.field("image").is_empty()).then(||view!{<figure class="container cms-intro-photo"><img src=item.image() alt=item.field("image_alt")/><figcaption>{item.field("credit")}</figcaption></figure>})}
                {(!item.field("button_url").is_empty()).then(||view!{<div class="container cms-intro-action"><a class="button button-green" href=url(&item.field("button_url"))>{item.field("button_text")}<Icon/></a></div>})}
            }.into_any();
        }
    }
    view! {<section class="page-intro container"><Eyebrow>{kicker}</Eyebrow><h1>{title}</h1><p class="intro-copy">{description}</p></section>}.into_any()
}

#[component]
pub fn Newsletter() -> impl IntoView {
    view!{<div class="newsletter"><div><p class="eyebrow">"LET THE TRAIL COME TO YOU"</p><h2>"Stay connected."</h2><p>"Stories, places, and ways to make a difference."</p></div><div class="newsletter-action"><a class="button button-light" href=url("/support/newsletter")>"Explore trail updates"<Icon/></a><p class="form-note">"Read The Tripod or register your interest."</p></div></div>}
}

#[component]
pub fn Footer() -> impl IntoView {
    view! {<footer class="site-footer"><div class="container"><Newsletter/>
        <div class="footer-grid"><div><Brand/><p class="footer-mission">"Connecting people to the history, landscapes, and living communities of the Iditarod National Historic Trail."</p></div>
            <div><p class="footer-label">"FIND YOUR TRAIL"</p><a href=url("/explore")>"Plan a trip"</a><a href=url("/stories")>"Stories & history"</a><a href=url("/resources")>"For educators & explorers"</a><a href=url("/store")>"Trail shop"</a></div>
            <div><p class="footer-label">"MAKE A CONNECTION"</p><a href=url("/about")>"About the Alliance"</a><a href=url("/get-involved")>"Membership & giving"</a><a href=url("/get-involved#volunteer")>"Volunteer"</a><a href="https://www.iditarod100.org/contact-us.html" target="_blank" rel="noopener noreferrer">"Contact the Alliance ↗"</a></div>
            <div class="footer-note"><span class="compass" aria-hidden="true">"✳"</span><p>"A trail for all seasons."<br/>"A legacy for everyone."</p><a href=url("/about") class="text-link">"The story of the Alliance"<Icon/></a></div>
        </div>
        <div class="footer-bottom"><span>"Iditarod Historic Trail Alliance"</span><span>"Seward to Nome. Then, now, and always."</span><a href=url("/support/privacy")>"Your details & privacy"</a><a href=url("/credits")>"Photo credits & sources"</a></div>
    </div></footer>}
}

#[component]
pub fn TrailDrawing() -> impl IntoView {
    view! {<svg class="trail-drawing" viewBox="0 0 420 470" fill="none" role="img" aria-label="Illustrated trail journey from Seward through Interior Alaska to Nome; not a navigation map">
        <path class="contour" d="M390 45C250 8 370 180 210 190S105 325 6 300M410 75C270 38 390 210 230 220S125 355 26 330M410 105C290 68 410 240 250 250S145 385 46 360M410 135C310 98 430 270 270 280S165 415 66 390M390 15C230-22 350 150 190 160S85 295-14 270"/>
        <path class="route-shadow" d="M325 407C270 370 318 310 257 288S226 214 193 192 173 129 128 126 106 79 55 74"/>
        <path class="route-path" d="M325 407C270 370 318 310 257 288S226 214 193 192 173 129 128 126 106 79 55 74"/>
        <circle cx="325" cy="407" r="6" fill="currentColor"/><circle cx="257" cy="288" r="4" fill="currentColor"/><circle cx="193" cy="192" r="4" fill="currentColor"/><circle cx="55" cy="74" r="6" fill="currentColor"/>
        <text x="272" y="446">"SEWARD"</text><text x="273" y="285">"SOUTHCENTRAL"</text><text x="55" y="194">"INTERIOR ALASKA"</text><text x="40" y="48">"NOME"</text>
        <text x="340" y="46" class="north">"N ↑"</text>
    </svg>}
}
