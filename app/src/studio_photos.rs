use super::{ctx, update_field, Edit, EditField};
use leptos::prelude::*;

#[derive(Clone)]
struct Photo { file: String, name: String, alt: String, credit: String }
fn collection() -> Vec<Photo> {
    let mut photos=vec![
        Photo{file:"/images/dawn.webp".into(),name:"Winter light".into(),alt:"Winter landscape along the Iditarod National Historic Trail".into(),credit:"Bob Wick / Bureau of Land Management".into()},
        Photo{file:"/images/summer.webp".into(),name:"Chugach landscape".into(),alt:"Meares Glacier and Unakwik Inlet in Chugach National Forest".into(),credit:"USDA Forest Service Alaska Region / CC BY 2.0".into()},
        Photo{file:"/images/winter.webp".into(),name:"A winter journey".into(),alt:"Musher and dog team near Tripod Flats Public Shelter Cabin".into(),credit:"Bureau of Land Management".into()},
        Photo{file:"/images/cycling.webp".into(),name:"Winter cycling".into(),alt:"Cyclists traveling a snowy winter trail".into(),credit:"Bob Wick / Bureau of Land Management".into()},
        Photo{file:"/images/dogs.webp".into(),name:"Sled dogs".into(),alt:"A sled dog team resting in the snow".into(),credit:"Bob Wick / Bureau of Land Management".into()},
        Photo{file:"/images/waterfront.webp".into(),name:"Resurrection Bay".into(),alt:"Resurrection Bay seen from Seward".into(),credit:"NPS / Max Odland".into()},
    ];
    let mut editorial:Vec<serde_json::Value>=serde_json::from_str(include_str!("../public/images/editorial/manifest.json")).unwrap_or_default();
    editorial.extend(serde_json::from_str::<Vec<serde_json::Value>>(include_str!("../public/images/explorer/manifest.json")).unwrap_or_default());
    photos.extend(editorial.into_iter().map(|p|{let f=|k:&str|p[k].as_str().unwrap_or_default().to_owned();Photo{file:f("file"),name:f("caption"),alt:f("alt"),credit:f("credit")}}));photos
}

fn focus(draft:&crate::studio_model::Draft,key:&str)->u8{draft.field(key).parse::<u8>().unwrap_or(50).min(100)}

#[component]
pub(super) fn FocusControls(state: Edit, #[prop(default=false)] summer: bool) -> impl IntoView {
    let user=ctx().session.get_value().user.id;
    let x=if summer{"summer_x"}else{"image_x"};let y=if summer{"summer_y"}else{"image_y"};
    let stored=StoredValue::new(state.clone());
    view!{<fieldset class="focus-controls"><legend>{if summer{"Summer photo focus"}else{"Photo focus"}}</legend><p class="small-note">"Keep the important part in view as the photo adapts to different screens. Your original file stays unchanged."</p>
        {[ (x,"Horizontal focus","Left","Right"),(y,"Vertical focus","Top","Bottom") ].into_iter().map(|(key,label,start,end)|{let read=state.clone();let write=state.clone();let output=state.clone();view!{<div class="focus-range"><label for=key>{if summer{format!("Summer {}",label.to_lowercase())}else{label.to_owned()}}<output for=key>{move||format!("{}%",focus(&output.get().draft,key))}</output></label><input id=key type="range" min="0" max="100" step="1" prop:value=move||focus(&read.get().draft,key).to_string() on:input=move|e|update_field(&write,key,event_target_value(&e),user)/><div class="focus-range-labels"><span>{start}</span><span>{end}</span></div></div>}}).collect_view()}
        <button type="button" class="text-button" on:click=move|_|{update_field(&stored.get_value(),x,"50".into(),user);update_field(&stored.get_value(),y,"50".into(),user);}>"Center photo"</button>
    </fieldset>}
}

#[component]
pub(super) fn PhotoControls(state: Edit, media: Vec<crate::studio_model::Media>, allow_layout: bool) -> impl IntoView {
    let user=ctx().session.get_value().user.id;
    let stored=StoredValue::new(state.clone());
    let photos=collection();
    view!{<section class="editor-panel photo-editor"><div class="photo-editor-heading"><span class="eyebrow">"MAKE ROOM FOR THE LANDSCAPE"</span><h2>"Photos & presentation"</h2><p class="small-note">"Choose a photograph, then set its focus. Check the full page with Preview before publishing."</p></div>
        <div class="photo-selection" class:portrait=move||stored.get_value().get().draft.field("image_layout")=="tall">{move||{let d=stored.get_value().get().draft;let src=d.field("image");if src.is_empty(){view!{<div class="photo-placeholder">"Choose a photo below"</div>}.into_any()}else{view!{<img src=src alt=d.field("image_alt") style=format!("object-position:{}% {}%",focus(&d,"image_x"),focus(&d,"image_y"))/><span class="crop-preview-label">"CROP PREVIEW"</span>}.into_any()}}}</div>
        {allow_layout.then(||view!{<div class="field"><label for="image-layout">"Feature photo shape"</label><select id="image-layout" prop:value=move||stored.get_value().get().draft.field("image_layout") on:change=move|e|update_field(&stored.get_value(),"image_layout",event_target_value(&e),user)><option value="">"Wide landscape"</option><option value="tall">"Taller photograph"</option></select><small>"Applies to the feature photograph on this page. Cards keep their own proportions."</small></div>})}
        <FocusControls state=state.clone()/>
        <details class="photo-library" open><summary>"Choose from the trail collection"<span>{format!("{} photographs",photos.len())}</span></summary><p class="small-note">"Selecting a collection photo fills in its description and credit. You can refine the description below."</p><div class="curated-photos">{photos.into_iter().map(|p|{let current=stored;let active=p.file.clone();view!{<button type="button" class="curated-photo" aria-label=format!("Choose {}",p.name) aria-pressed=move||(current.get_value().get().draft.field("image")==active).to_string() on:click=move|_|{for(key,value)in[("image",p.file.clone()),("image_alt",p.alt.clone()),("credit",p.credit.clone()),("image_x","50".into()),("image_y","50".into())]{update_field(&stored.get_value(),key,value,user);}}><img src=p.file.clone() alt="" loading="lazy"/><span>{p.name.clone()}</span></button>}}).collect_view()}</div></details>
        {(!media.is_empty()).then(||view!{<details class="photo-library"><summary>"Your uploaded photos"</summary><div class="curated-photos">{media.into_iter().filter(|m|m.mime.starts_with("image/")).map(|m|{let file=m.url.clone();view!{<button type="button" class="curated-photo" aria-label=format!("Choose {}",m.name) on:click=move|_|{for(key,value)in[("image",file.clone()),("image_alt",String::new()),("credit",String::new()),("image_x","50".into()),("image_y","50".into())]{update_field(&stored.get_value(),key,value,user);}}><img src=m.url alt="" loading="lazy"/><span>{m.name.clone()}</span></button>}}).collect_view()}</div></details>})}
        <EditField state=state.clone() key="image" label="Photo address" hint="Choose above, or paste a photo address from Photos & files."/><EditField state=state.clone() key="image_alt" label="Photo description" hint="Describe what matters for someone who cannot see the photograph."/><EditField state=state.clone() key="credit" label="Photo credit" hint="Keep the photographer or organization credit with the image."/>
        <Show when=move||{let d=stored.get_value().get().draft;!d.field("image").is_empty()&&d.field("image_alt").trim().is_empty()}><p class="photo-reminder">"Add a photo description before publishing so everyone can share the story."</p></Show>
    </section>}
}

#[component]
pub(super) fn SummerPhoto(state: Edit) -> impl IntoView {
    let stored=StoredValue::new(state.clone());
    view!{<div class="summer-presentation"><EditField state=state.clone() key="summer_image" label="Summer photo address"/><EditField state=state.clone() key="summer_credit" label="Summer photo credit"/><div class="photo-selection">{move||{let d=stored.get_value().get().draft;let src=d.field("summer_image");(!src.is_empty()).then(||view!{<img src=src alt="Summer landscape crop preview" style=format!("object-position:{}% {}%",focus(&d,"summer_x"),focus(&d,"summer_y"))/><span class="crop-preview-label">"SUMMER CROP PREVIEW"</span>})}}</div><FocusControls state summer=true/></div>}
}
