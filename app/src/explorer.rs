//! A geographic field guide. Geometry is fixed to sourced locations; published
//! place content can override the editorial introduction and seasonal ideas.
use crate::{cms, components::Icon, paths::url};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::Deserialize;

pub const STOPS: &[(&str, &str)] = &[
    ("seward", "Seward"),
    ("crow-pass", "Crow Pass"),
    ("knik", "Knik"),
    ("iditarod", "Iditarod"),
    ("unalakleet", "Unalakleet"),
    ("nome", "Nome"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Season {
    Winter,
    Summer,
}
impl Season {
    pub fn label(self) -> &'static str {
        if self == Self::Summer {
            "Summer"
        } else {
            "Winter"
        }
    }
    pub fn key(self) -> &'static str {
        if self == Self::Summer {
            "summer"
        } else {
            "winter"
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "summer" => Some(Self::Summer),
            "winter" => Some(Self::Winter),
            _ => None,
        }
    }
}
#[derive(Clone, Copy)]
pub struct TrailState {
    pub season: RwSignal<Season>,
    pub selected: RwSignal<usize>,
}
pub fn provide_trail_state() {
    let query = use_query_map();
    let initial = query.get_untracked();
    let state = TrailState {
        season: RwSignal::new(
            initial
                .get("season")
                .as_deref()
                .and_then(Season::parse)
                .unwrap_or(Season::Winter),
        ),
        selected: RwSignal::new(
            initial
                .get("place")
                .and_then(|id| STOPS.iter().position(|(key, _)| *key == id))
                .unwrap_or(0),
        ),
    };
    provide_context(state);
    Effect::new(move |_| {
        let q = query.get();
        if let Some(s) = q.get("season").as_deref().and_then(Season::parse) {
            state.season.set(s);
        }
        if let Some(i) = q
            .get("place")
            .and_then(|id| STOPS.iter().position(|(key, _)| *key == id))
        {
            state.selected.set(i);
        }
    });
}
pub fn trail_state() -> TrailState {
    use_context().unwrap_or_else(|| TrailState {
        season: RwSignal::new(Season::Winter),
        selected: RwSignal::new(0),
    })
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Experience {
    pub label: String,
    pub activities: Vec<String>,
    pub note: String,
    pub url: String,
    pub link_label: String,
}
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Stop {
    pub id: String,
    pub name: String,
    pub region: String,
    pub subtitle: String,
    pub x: f64,
    pub y: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub image: String,
    pub thumbnail: String,
    pub alt: String,
    pub credit: String,
    pub caption: String,
    pub history: String,
    pub history_url: String,
    pub page: String,
    pub summer: Experience,
    pub winter: Experience,
    #[serde(default)]
    pub position: String,
}
impl Stop {
    fn experience(&self, season: Season) -> Experience {
        if season == Season::Summer {
            self.summer.clone()
        } else {
            self.winter.clone()
        }
    }
    fn coordinate_label(&self) -> String {
        format!("{:.2} N / {:.2} W", self.latitude, self.longitude.abs())
    }
}
pub fn stops_with_content(content: Vec<cms::Content>) -> Vec<Stop> {
    let mut stops: Vec<Stop> =
        serde_json::from_str(include_str!("../public/maps/explorer-places.json"))
            .expect("valid explorer places");
    for stop in &mut stops {
        if let Some(item) = content
            .iter()
            .find(|item| item.kind == "place" && item.field("map_stop") == stop.id)
        {
            if !item.summary.trim().is_empty() {
                stop.subtitle = item.summary.clone();
            }
            if !item.field("trail_history").trim().is_empty() {
                stop.history = item.field("trail_history");
            }
            if !item.field("history_url").trim().is_empty() {
                stop.history_url = item.field("history_url");
            }
            if !item.field("image").is_empty() {
                stop.image = item.image();
                stop.thumbnail = stop.image.clone();
                stop.alt = item.field("image_alt");
                stop.credit = item.field("credit");
                stop.caption = item.field("photo_caption");
                stop.position = item.image_position();
            }
            for (key, experience) in [
                ("summer_activities", &mut stop.summer),
                ("winter_activities", &mut stop.winter),
            ] {
                let activities: Vec<_> = item
                    .field(key)
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_owned)
                    .take(6)
                    .collect();
                if !activities.is_empty() {
                    experience.activities = activities;
                }
            }
            if !item.field("url").is_empty() {
                stop.summer.url = item.field("url");
                stop.winter.url = item.field("url");
                stop.summer.link_label = "Visitor information".into();
                stop.winter.link_label = "Visitor information".into();
            }
            stop.page = item.path.clone();
        }
    }
    stops
}

#[component]
pub fn ExplorerSeason() -> impl IntoView {
    let state = trail_state();
    view! {<div class="explorer-seasons" role="group" aria-label="Explorer season">{[Season::Winter,Season::Summer].into_iter().map(|s|view!{<button type="button" class:active=move||state.season.get()==s aria-pressed=move||(state.season.get()==s).to_string() on:click=move|_|state.season.set(s)><Icon name=if s==Season::Winter{"snow"}else{"sun"}/>{s.label()}</button>}).collect_view()}</div>}
}

fn focus_mobile_detail() {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        if let Some(w) = web_sys::window() {
            if w.inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1200.)
                < 900.
            {
                leptos::prelude::request_animation_frame(move || {
                    if let Some(e) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.get_element_by_id("trail-chapter"))
                    {
                        e.scroll_into_view();
                        if let Ok(el) = e.dyn_into::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    }
                });
            }
        }
    }
}
fn select_place(state: TrailState, index: usize) {
    state.selected.set(index);
    focus_mobile_detail();
}

#[component]
pub fn TrailExplorer() -> impl IntoView {
    let state = trail_state();
    let stops = StoredValue::new(stops_with_content(cms::items("place")));
    let selected = Memo::new(move |_| stops.get_value()[state.selected.get().min(5)].clone());
    let regional = RwSignal::new(matches!(state.selected.get_untracked(), 1 | 2));
    let mobile_map = RwSignal::new(false);
    Effect::new(move |_| match state.selected.get() {
        1 | 2 => regional.set(true),
        3.. => regional.set(false),
        _ => (),
    });
    view! {
        <section class="trail-explorer" id="trail-atlas" class:explorer-summer=move||state.season.get()==Season::Summer aria-labelledby="atlas-title">
            <div class="container explorer-heading"><div><span class="section-marker"><span>"01"</span>"YOUR WAY INTO ALASKA"</span><h2 id="atlas-title">"One trail."<br/><em>"A thousand ways in."</em></h2></div><div><p>"Follow the coast, cross the mountains, and meet the places that carry the trail's story."</p><span class="explorer-heading-note">"SIX PLACES / SEWARD TO NOME"</span></div></div>
            <div class="container explorer-shell">
                <div class="explorer-toolbar"><div><span class="explorer-control-label">"Choose your season"</span><ExplorerSeason/></div><p class="explorer-season-intro" aria-live="polite">{move||if state.season.get()==Season::Summer{"Summer walks, coastal places, and stories to discover."}else{"Winter heritage, community connections, and journeys to plan."}}</p><div class="explorer-mobile-tabs" role="group" aria-label="Browse places"><button class:active=move||!mobile_map.get() aria-pressed=move||(!mobile_map.get()).to_string() on:click=move|_|mobile_map.set(false)>"List"<span>"6"</span></button><button class:active=mobile_map aria-pressed=move||mobile_map.get().to_string() on:click=move|_|mobile_map.set(true)>"Map"<Icon name="pin"/></button></div></div>
                <div class="explorer-layout"><div class="explorer-browser" class:show-mobile-map=mobile_map>
                    <div class="explorer-map-panel"><div class="explorer-map-topline"><span>"ALASKA / HISTORIC TRAIL"</span><div class="explorer-map-views" role="group" aria-label="Map extent"><button class:active=move||!regional.get() aria-pressed=move||(!regional.get()).to_string() on:click=move|_|regional.set(false)>"Whole trail"</button><button class:active=regional aria-pressed=move||regional.get().to_string() on:click=move|_|regional.set(true)>"Southcentral"<Icon name="search"/></button></div></div>
                        <div class="explorer-map" class:regional=regional>
                            <svg class="explorer-geography" fill="none" viewBox=move||if regional.get(){"590 420 240 186.6667"}else{"0 0 900 700"} role="img" aria-label=move||if regional.get(){"Geographic detail of Seward, Crow Pass and Knik in Southcentral Alaska"}else{"Alaska coastline and rivers with the historic Iditarod trail alignment from Seward to Nome"} inner_html={let svg=include_str!("../public/maps/explorer-base.svg");svg[svg.find('>').unwrap()+1..svg.rfind("</svg>").unwrap()].to_string()}/>
                            {stops.get_value().into_iter().enumerate().map(|(i,p)|view!{<Show when=move||if regional.get(){i<3}else{i!=1&&i!=2}><button type="button" class="explorer-pin" class:active=move||state.selected.get()==i style=move||{let (x,y,w,h)=if regional.get(){(590.,420.,240.,186.6667)}else{(0.,0.,900.,700.)};format!("left:{}%;top:{}%",(p.x-x)/w*100.,(p.y-y)/h*100.)} data-place=p.id.clone() aria-pressed=move||(state.selected.get()==i).to_string() aria-controls="trail-chapter" on:click=move|_|select_place(state,i)><span class="explorer-pin-dot">{format!("0{}",i+1)}</span><span class="explorer-pin-label">{p.name.clone()}</span></button></Show>}).collect_view()}
                            <Show when=move||!regional.get()><button type="button" class="explorer-pin explorer-cluster" class:active=move||matches!(state.selected.get(),1|2) style="left:77.16%;top:71.52%" title="Explore Southcentral: Crow Pass and Knik" on:click=move|_|regional.set(true)><span class="explorer-pin-dot">"2"</span><span class="explorer-pin-label">"Southcentral"<small>"ZOOM TO TWO PLACES"</small></span></button></Show>
                            <span class="explorer-north" aria-hidden="true">"N"<span>"↑"</span></span>
                        </div>
                        <div class="explorer-map-legend"><span><i/>"Historic main trail"</span><span>"Orientation map"</span><a href=url("/credits#map-sources")>"Sources"<Icon name="external"/></a></div>
                    </div>
                    <div class="explorer-place-list" role="group" aria-label="Places along the trail">{stops.get_value().into_iter().enumerate().map(|(i,p)|{let seasonal=p.clone();view!{<button type="button" class="explorer-place-option" class:active=move||state.selected.get()==i aria-pressed=move||(state.selected.get()==i).to_string() data-place=p.id.clone() aria-controls="trail-chapter" on:click=move|_|select_place(state,i)><img src=url(&p.thumbnail) alt="" loading="lazy" width="120" height="100"/><span><small>{format!("0{} / {}",i+1,p.region)}</small><strong>{p.name.clone()}</strong><span class="explorer-list-season">{move||seasonal.experience(state.season.get()).label}</span></span><Icon/></button>}}).collect_view()}</div>
                </div>
                <article class="explorer-detail" id="trail-chapter" tabindex="-1" aria-labelledby="explorer-place-title">
                    <p class="sr-only" role="status">{move||format!("{} selected. {} ideas.",selected.get().name,state.season.get().label())}</p>
                    {move||{let p=selected.get();let experience=p.experience(state.season.get());view!{
                        <figure class="explorer-place-photo"><img src=url(&p.image) style=p.position.clone() alt=p.alt.clone() width="1200" height="800"/><figcaption><span>{p.caption.clone()}</span><a href=url("/credits#explorer-photography") aria-label=format!("Photo credit: {}",p.credit)><Icon name="external"/></a></figcaption></figure>
                        <div class="explorer-detail-copy"><div class="explorer-place-meta"><span>{p.region.clone()}</span><span>{p.coordinate_label()}</span></div><h3 id="explorer-place-title">{p.name.clone()}<span>"."</span></h3><p class="explorer-place-deck">{p.subtitle.clone()}</p>
                            <div class="explorer-history"><h4>"A place in the story"</h4><p>{p.history.clone()}</p><a href=p.history_url.clone() target="_blank" rel="noopener noreferrer">"Explore the history"<Icon name="external"/></a></div>
                            <div class="explorer-experience"><div><Icon name=if state.season.get()==Season::Summer{"sun"}else{"snow"}/><h4>{format!("{} / {}",state.season.get().label(),experience.label)}</h4></div><ul>{experience.activities.into_iter().map(|a|view!{<li>{a}</li>}).collect_view()}</ul><p>{experience.note}</p><a class="text-link" href=experience.url target="_blank" rel="noopener noreferrer">{experience.link_label}<Icon name="external"/></a></div>
                            <div class="explorer-useful-links"><a href=url(&p.page)>"Continue exploring"<Icon/></a><a href=url(&format!("/explore?place={}&season={}#trail-atlas",p.id,state.season.get().key()))>"Link to this place"<Icon name="external"/></a></div>
                        </div>
                    }}}
                    <div class="explorer-place-pagination"><button type="button" aria-label="Previous place" on:click=move|_|select_place(state,(state.selected.get_untracked()+5)%6)><span aria-hidden="true">"←"</span>"Previous"</button><span>{move||format!("0{} / 06",state.selected.get()+1)}</span><button type="button" aria-label="Next place" on:click=move|_|select_place(state,(state.selected.get_untracked()+1)%6)>"Next"<Icon/></button></div>
                </article></div>
            </div>
            <div class="container explorer-footnote"><p>"A historic connection, not a continuous year-round walking route. The map shows the main historic alignment, not every connecting trail or today's race route. Plan with local guidance and respect land ownership."</p><a href=url("/resources")>"Maps & planning resources"<Icon/></a></div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn place_coordinates_match_the_authoritative_map_and_have_seasonal_guidance() {
        let data: serde_json::Value =
            serde_json::from_str(include_str!("../public/maps/map-sources.json")).unwrap();
        let pins = data["views"]["trail-orientation"]["pins"]
            .as_array()
            .unwrap();
        for p in stops_with_content(vec![]) {
            let pin = pins.iter().find(|pin| pin["id"] == p.id).unwrap();
            assert!((pin["x"].as_f64().unwrap() - p.x).abs() < 0.01);
            assert!((pin["y"].as_f64().unwrap() - p.y).abs() < 0.01);
            assert!(!p.summer.activities.is_empty() && !p.winter.activities.is_empty());
            assert_ne!(p.summer.note, p.winter.note);
        }
    }
    #[test]
    fn editorial_override_keeps_geography_and_other_stops_intact() {
        let original = stops_with_content(vec![]);
        let item = cms::Content {
            kind: "place".into(),
            path: "/places/updated-nome".into(),
            summary: "A new introduction".into(),
            fields: std::collections::HashMap::from([
                ("map_stop".into(), "nome".into()),
                ("trail_history".into(), "Reviewed history".into()),
                (
                    "summer_activities".into(),
                    "Museum visit\nCoastal photography".into(),
                ),
            ]),
            ..Default::default()
        };
        let updated = stops_with_content(vec![item]);
        assert_eq!(updated[5].history, "Reviewed history");
        assert_eq!(updated[5].x, original[5].x);
        assert_eq!(updated[0].history, original[0].history);
        assert_eq!(updated[5].winter.note, original[5].winter.note);
    }
}
