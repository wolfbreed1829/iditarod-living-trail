use crate::{components::*, paths::url};
use leptos::prelude::*;

// Each observer is owned by its component and disconnected when the route leaves.
// SSR content stays visible until an observer has successfully attached.
#[cfg(feature = "hydrate")]
struct RevealObserver {
    observer: web_sys::IntersectionObserver,
    _callback:
        wasm_bindgen::closure::Closure<dyn FnMut(js_sys::Array, web_sys::IntersectionObserver)>,
}

#[cfg(feature = "hydrate")]
impl Drop for RevealObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

#[component]
pub fn Reveal(children: Children) -> impl IntoView {
    let node = NodeRef::<leptos::html::Div>::new();
    let visible = RwSignal::new(false);
    let observed = RwSignal::new(false);
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::{closure::Closure, JsCast};
        let handle = StoredValue::new_local(None::<RevealObserver>);
        Effect::new(move |_| {
            let Some(element) = node.get() else { return };
            let callback = Closure::wrap(Box::new(
                move |entries: js_sys::Array, observer: web_sys::IntersectionObserver| {
                    if entries.iter().any(|entry| {
                        entry
                            .unchecked_into::<web_sys::IntersectionObserverEntry>()
                            .is_intersecting()
                    }) {
                        visible.set(true);
                        observer.disconnect();
                    }
                },
            )
                as Box<dyn FnMut(js_sys::Array, web_sys::IntersectionObserver)>);
            let options = web_sys::IntersectionObserverInit::new();
            options.set_root_margin("0px 0px -24px 0px");
            options.set_threshold_f64(0.04);
            if let Ok(observer) = web_sys::IntersectionObserver::new_with_options(
                callback.as_ref().unchecked_ref(),
                &options,
            ) {
                observer.observe(&element);
                observed.set(true);
                handle.set_value(Some(RevealObserver {
                    observer,
                    _callback: callback,
                }));
            }
        });
    }
    view! {
        <div node_ref=node class="motion-reveal" class:is-observed=move || observed.get() class:is-visible=move || visible.get()>
            {children()}
        </div>
    }
}

#[component]
pub fn SeasonalHero(paused: RwSignal<bool>) -> impl IntoView {
    let trail=crate::explorer::trail_state();
    let summer = Memo::new(move |_|trail.season.get()==crate::explorer::Season::Summer);
    let content = crate::cms::page("home");
    let title = content
        .as_ref()
        .map(|i| i.title.clone())
        .unwrap_or("A trail through Alaska.".into());
    let accent = content
        .as_ref()
        .map(|i| i.field("headline_accent"))
        .unwrap_or("A connection across generations.".into());
    let description = content.as_ref().map(|i| i.summary.clone()).unwrap_or(
        "Follow the landscapes, stories, and communities that make this trail a living legacy."
            .into(),
    );
    let eyebrow = content
        .as_ref()
        .map(|i| i.field("category"))
        .unwrap_or("ALASKA'S IDITAROD NATIONAL HISTORIC TRAIL".into());
    let winter_photo = content
        .as_ref()
        .map(|i| i.image())
        .filter(|s| !s.is_empty())
        .unwrap_or(url("/images/dawn.webp"));
    let summer_photo = content
        .as_ref()
        .map(|i| url(&i.field("summer_image")))
        .filter(|s| !s.is_empty())
        .unwrap_or(url("/images/summer.webp"));
    let action = content
        .as_ref()
        .map(|i| i.field("button_text"))
        .filter(|s| !s.is_empty())
        .unwrap_or("Find your trail".into());
    let action_url = content
        .as_ref()
        .map(|i| url(&i.field("button_url")))
        .filter(|s| !s.is_empty())
        .unwrap_or(url("/explore"));
    let winter_credit = content
        .as_ref()
        .map(|i| i.field("credit"))
        .filter(|s| !s.is_empty())
        .unwrap_or("Iditarod National Historic Trail · Bob Wick / BLM".into());
    let summer_credit = content
        .as_ref()
        .map(|i| i.field("summer_credit"))
        .filter(|s| !s.is_empty())
        .unwrap_or("Chugach National Forest · USDA Forest Service".into());
    let winter_position=content.as_ref().map(|i|i.image_position()).unwrap_or_else(||"object-position:50% 50%".into());
    let summer_x=content.as_ref().and_then(|i|i.field("summer_x").parse::<u8>().ok()).unwrap_or(50).min(100);
    let summer_y=content.as_ref().and_then(|i|i.field("summer_y").parse::<u8>().ok()).unwrap_or(50).min(100);
    let summer_position=format!("object-position:{summer_x}% {summer_y}%");
    let winter_srcset=crate::photography::by_file(&winter_photo).map(|p|p.srcset()).unwrap_or_default();
    let summer_srcset=crate::photography::by_file(&summer_photo).map(|p|p.srcset()).unwrap_or_default();
    let winter_photo=StoredValue::new(winter_photo);let summer_photo=StoredValue::new(summer_photo);
    let winter_srcset=StoredValue::new(winter_srcset);let summer_srcset=StoredValue::new(summer_srcset);
    let winter_position=StoredValue::new(winter_position);let summer_position=StoredValue::new(summer_position);
    view! {
        <section class="hero cinematic-hero" class:is-summer=move || summer.get()>
            <div class="hero-landscape" aria-hidden="true">
                <img class="hero-image is-active" class:hero-winter=move||!summer.get() class:hero-summer=move||summer.get()
                    src=move||if summer.get(){summer_photo.get_value()}else{winter_photo.get_value()}
                    srcset=move||if summer.get(){summer_srcset.get_value()}else{winter_srcset.get_value()} sizes="100vw"
                    style=move||if summer.get(){summer_position.get_value()}else{winter_position.get_value()}
                    alt="" fetchpriority="high" decoding="async" width="2200" height="1100"/>
            </div>
            <div class="hero-shade"/>
            <div class="weather-layer" aria-hidden="true">
                <div class="snowfall">{(0..22).map(|i| {
                    let style = format!("--x:{}%;--drift:{}px;--duration:{}s;--delay:-{}s;--size:{}px;--alpha:{}", (i*43+7)%100, 20+(i*17)%70, 13+(i*7)%14, (i*11)%27, 1+(i%3), 0.22+f64::from(i%4)*0.1);
                    view! {<span class="snowflake" style=style/>}
                }).collect_view()}</div>
                <div class="summer-light"/>
            </div>
            <div class="hero-content container">
                <p class="eyebrow hero-intro"><span class="status-dot"/>{eyebrow}</p>
                <h1><span class="headline-line"><span>{title}</span></span><em><span class="headline-line"><span>{accent}</span></span></em></h1>
                <p class="hero-description">{description}</p>
                <div class="hero-buttons"><a class="button button-light" href=action_url>{action}<Icon/></a><a class="button button-outline-light" href=url("/stories")>"Discover the story"</a></div>
            </div>
            <div class="hero-side"><span>"61° NORTH · A WORLD OF CONNECTION"</span><span class="hero-side-line"/></div>
            <div class="hero-bottom container">
                <a href="#discover" class="scroll-prompt"><span class="scroll-circle"><Icon name="down"/></span>"A little further. A little closer."</a>
                <div class="scene-controls">
                    <button class="motion-toggle" aria-pressed=move || paused.get().to_string() aria-label=move || if paused.get(){"Resume motion"}else{"Pause motion"} on:click=move |_|paused.update(|value| *value = !*value)>
                        {move || if paused.get(){view!{<Icon name="play"/>}}else{view!{<Icon name="pause"/>}}}
                        <span>{move || if paused.get(){"Motion paused"}else{"Pause motion"}}</span>
                    </button>
                    <span class="reduced-motion-note">"Reduced motion"</span>
                    <div class="season-switch" role="group" aria-label="Landscape season">
                        <span class="season-indicator" aria-hidden="true"/>
                        <button class:active=move || !summer.get() aria-pressed=move || (!summer.get()).to_string() on:click=move |_|trail.season.set(crate::explorer::Season::Winter)><Icon name="snow"/>"Winter"</button>
                        <button class:active=move || summer.get() aria-pressed=move || summer.get().to_string() on:click=move |_|trail.season.set(crate::explorer::Season::Summer)><Icon name="sun"/>"Summer"</button>
                    </div>
                </div>
            </div>
            <a class="hero-credit" href=url("/credits")>{move || if summer.get(){summer_credit.clone()}else{winter_credit.clone()}}</a>
        </section>
    }
}

#[derive(Clone, Copy)]
struct Chapter {
    name: &'static str,
    label: &'static str,
    title: &'static str,
    story: &'static str,
    link: &'static str,
    action: &'static str,
    x: u32,
    y: u32,
    progress: f64,
}

const CHAPTERS: [Chapter; 4] = [
    Chapter { name: "Seward", label: "01 / THE SOUTHERN GATEWAY", title: "Where the mountains meet the sea.", story: "Begin on the shores of Resurrection Bay. In Seward, the trail's story connects a coastal community to the vast landscapes of Alaska.", link: "/places/seward", action: "Explore Seward", x: 325, y: 407, progress: 0.015 },
    Chapter { name: "Southcentral", label: "02 / INTO THE LANDSCAPE", title: "A little further into the wild.", story: "Mountains, forests, and changing seasons. Discover the southern trail's connection to Chugach National Forest and find your own way into its story.", link: "/places/chugach", action: "Discover Chugach", x: 257, y: 288, progress: 0.32 },
    Chapter { name: "Interior", label: "03 / THE LIVING CONNECTION", title: "Carried forward, generation to generation.", story: "Long before the gold rush, Alaska Native travel networks connected people and places. Explore the knowledge, communities, and histories woven into the trail.", link: "/stories/a-living-connection", action: "Meet the living trail", x: 193, y: 192, progress: 0.57 },
    Chapter { name: "Nome", label: "04 / A STORY THAT CONTINUES", title: "The trail ends. The connection stays.", story: "On the Bering Sea coast, Nome is the trail's northern gateway. Every journey is part of a larger story, kept alive by the people who care for it.", link: "/stories", action: "Keep exploring the stories", x: 55, y: 74, progress: 1.0 },
];

#[component]
pub fn TrailJourney() -> impl IntoView {
    let selected = RwSignal::new(0usize);
    view! {
        <section class="journey-section interactive-journey" aria-labelledby="journey-title">
            <div class="container">
                <Reveal><div class="journey-heading"><Eyebrow>"A JOURNEY THROUGH CONNECTION"</Eyebrow><h2 id="journey-title">"More than a route. "<em>"A living thread."</em></h2><p>"Four places. One connected story. Choose a chapter and follow the trail."</p></div></Reveal>
                <Reveal><div class="journey-grid">
                    <div class="journey-art">
                        <svg class="trail-drawing interactive-route" viewBox="0 0 420 470" fill="none" role="img" aria-label=move || format!("Illustrated journey from Seward to Nome. Selected chapter: {}. Not a navigation map.", CHAPTERS[selected.get()].name)>
                            <path class="contour" d="M390 45C250 8 370 180 210 190S105 325 6 300M410 75C270 38 390 210 230 220S125 355 26 330M410 105C290 68 410 240 250 250S145 385 46 360M410 135C310 98 430 270 270 280S165 415 66 390M390 15C230-22 350 150 190 160S85 295-14 270"/>
                            <path class="route-shadow" d="M325 407C270 370 318 310 257 288S226 214 193 192 173 129 128 126 106 79 55 74"/>
                            <path class="route-guide" pathLength="1" d="M325 407C270 370 318 310 257 288S226 214 193 192 173 129 128 126 106 79 55 74"/>
                            <path class="route-progress" pathLength="1" style=move || format!("--route-progress:{}",CHAPTERS[selected.get()].progress) d="M325 407C270 370 318 310 257 288S226 214 193 192 173 129 128 126 106 79 55 74"/>
                            {CHAPTERS.into_iter().enumerate().map(|(i, chapter)| view! {
                                <g class="route-stop" class:is-current=move || selected.get()==i class:is-traveled=move || {selected.get()>=i}>
                                    <circle class="stop-halo" cx=chapter.x cy=chapter.y r="17"/>
                                    <circle class="stop-ring" cx=chapter.x cy=chapter.y r="11"/>
                                    <circle class="stop-core" cx=chapter.x cy=chapter.y r="4"/>
                                </g>
                            }).collect_view()}
                            <text x="272" y="446">"SEWARD"</text><text x="275" y="285">"SOUTHCENTRAL"</text><text x="55" y="194">"INTERIOR"</text><text x="40" y="48">"NOME"</text><text x="340" y="46" class="north">"N ↑"</text>
                        </svg>
                        <span class="drawing-caption">"An illustrated journey. Not a navigation map."</span>
                    </div>
                    <div class="journey-copy">
                        <div class="chapter-controls" role="group" aria-label="Trail chapters">{CHAPTERS.into_iter().enumerate().map(|(i, chapter)| view! {
                            <button aria-pressed=move || (selected.get()==i).to_string() aria-controls="trail-chapter" class:active=move || selected.get()==i on:click=move |_|selected.set(i)><span>{format!("0{}",i+1)}</span>{chapter.name}</button>
                        }).collect_view()}</div>
                        <div id="trail-chapter" class="chapter-stage" aria-live="polite" aria-atomic="true">
                            <For each=move || vec![selected.get()] key=|index| *index children=move |index| {
                                let chapter = CHAPTERS[index];
                                view! {<div class="chapter-content"><span class="eyebrow">{chapter.label}</span><h3>{chapter.title}</h3><p>{chapter.story}</p><a class="text-link" href=url(chapter.link)>{chapter.action}<Icon/></a></div>}
                            }/>
                        </div>
                        <div class="journey-footnote"><span>"THEN. NOW. ALWAYS."</span><span class="small-line"/></div>
                    </div>
                </div></Reveal>
            </div>
        </section>
    }
}
