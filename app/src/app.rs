use crate::paths::{url, BASE_PATH};
use crate::{components::*, pages::*};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Meta, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#193c34"/>
                <meta name="robots" content="noindex, nofollow"/>
                <meta name="application-name" content="The Living Trail"/>
                <meta property="og:title" content="The Living Trail | Iditarod Historic Trail Alliance"/>
                <meta property="og:description" content="A trail through Alaska. A connection across generations. Discover the places, people, and history of Alaska's Iditarod National Historic Trail."/>
                <link rel="icon" type="image/svg+xml" href=url("/favicon.svg")/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options root=BASE_PATH/>
                <MetaTags/>
            </head>
            <body><App/></body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(Cart(RwSignal::new([0u32; 3])));
    view! {
        <Stylesheet id="leptos" href=url("/pkg/living-trail.css")/>
        <Stylesheet id="cms-layout" href=url("/cms.css")/>
        <Stylesheet id="editorial" href=url("/design/editorial.css")/>
        <Stylesheet id="editorial-pages" href="/design/pages.css"/>
        <Stylesheet id="trail-explorer" href=url("/design/explorer.css")/>
        <Stylesheet id="support-journeys" href=url("/design/support.css")/>
        <Stylesheet id="interiors" href=url("/design/interiors.css")/>
        <Title text="The Living Trail | Iditarod Historic Trail Alliance"/>
        <Router base=BASE_PATH>
            <AppRouter/>
        </Router>
    }
}

#[component]
fn AppRouter() -> impl IntoView {
    crate::explorer::provide_trail_state();
    let location = leptos_router::hooks::use_location();
    view! {<Show when=move||location.pathname.get().starts_with("/admin") fallback=||view!{<crate::cms::CmsGate/>}>
        <Routes fallback=||view!{<crate::studio::Studio/>}>
            <Route path=path!("admin") view=crate::studio::Studio/>
            <Route path=path!("admin/*any") view=crate::studio::Studio/>
        </Routes>
    </Show>}
}

#[component]
pub fn SiteContent(data: Option<crate::cms::Catalog>) -> impl IntoView {
    let standalone = data.is_none();
    provide_context(crate::cms::CmsData(StoredValue::new(data)));
    view! {
            {standalone.then(||view!{<Meta name="description" content="Explore a living connection across Alaska. Discover the Iditarod National Historic Trail."/>})}
            <div class="field-journal"><a class="skip-link" href="#main">"Skip to content"</a>
            <Header/>
            <main id="main" tabindex="-1">
                <Routes fallback=|| view!{<NotFound/>}>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("admin") view=crate::studio::Studio/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("admin/*any") view=crate::studio::Studio/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("") view=crate::editorial::EditorialHome/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("explore") view=crate::cms::ManagedExplore/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("places/:slug") view=||view!{<crate::cms::ManagedDetail kind="place"/>}/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("stories") view=crate::cms::ManagedStories/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("stories/:slug") view=||view!{<crate::cms::ManagedDetail kind="story"/>}/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("get-involved") view=crate::cms::ManagedSupport/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("support/privacy") view=crate::support::Privacy/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("support/confirmation/:key") view=crate::support::Confirmation/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("support/:kind") view=crate::support::Journey/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("store") view=crate::editorial_pages::TrailShop/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("about") view=crate::cms::ManagedAbout/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("resources") view=crate::cms::ManagedResources/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("resources/:slug") view=||view!{<crate::cms::ManagedDetail kind="resource"/>}/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("events/:slug") view=||view!{<crate::cms::ManagedDetail kind="event"/>}/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("campaigns/:slug") view=||view!{<crate::cms::ManagedDetail kind="campaign"/>}/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("cms-preview/:token") view=crate::cms::CmsPreview/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("preview") view=crate::editorial_pages::ReviewGuide/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!("credits") view=crate::editorial::EditorialCredits/>
                    <Route ssr=leptos_router::SsrMode::Async path=path!(":slug") view=crate::cms::ManagedGeneric/>
                </Routes>
            </main>
            <Footer/></div>
    }
}
