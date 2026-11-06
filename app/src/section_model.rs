//! Approved content sections: text and photographs, never arbitrary markup or styles.
use crate::{cms::Content, studio_model::Draft};
use serde::{Deserialize, Serialize};

pub const FIELD: &str = "page_sections";
pub const MAX_SECTIONS: usize = 8;
pub const OPTIONS: &[(&str, &str, &str)] = &[
    ("text", "Editorial text", "A generous heading and readable paragraphs."),
    ("photo", "Captioned photograph", "A full-width photograph with its context and credit."),
    ("split", "Photo & story", "A photograph beside a short story or introduction."),
    ("callout", "Invitation", "A clear next step on a softly coloured background."),
];

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Section {
    pub id: u32,
    pub kind: String,
    #[serde(default)] pub hidden: bool,
    #[serde(default)] pub eyebrow: String,
    #[serde(default)] pub title: String,
    #[serde(default)] pub body: String,
    #[serde(default)] pub image: String,
    #[serde(default)] pub alt: String,
    #[serde(default)] pub caption: String,
    #[serde(default)] pub credit: String,
    #[serde(default)] pub source_url: String,
    #[serde(default="center")] pub x: u8,
    #[serde(default="center")] pub y: u8,
    #[serde(default)] pub reverse: bool,
    #[serde(default)] pub button_text: String,
    #[serde(default)] pub button_url: String,
}
fn center()->u8 {50}
impl Section {
    pub fn new(id:u32,kind:&str)->Self {
        Self{id,kind:kind.into(),hidden:false,eyebrow:String::new(),title:String::new(),body:String::new(),image:String::new(),alt:String::new(),caption:String::new(),credit:String::new(),source_url:String::new(),x:50,y:50,reverse:false,button_text:String::new(),button_url:String::new()}
    }
    pub fn label(&self)->&'static str {OPTIONS.iter().find(|(k,_,_)|*k==self.kind).map(|(_,l,_)|*l).unwrap_or("Section")}
    pub fn has_photo(&self)->bool {self.kind=="photo"||self.kind=="split"}
    pub fn content(&self)->Content {
        Content{fields:[("image",self.image.clone()),("image_alt",self.alt.clone()),("photo_caption",self.caption.clone()),("credit",self.credit.clone()),("image_x",self.x.to_string()),("image_y",self.y.to_string())].into_iter().map(|(k,v)|(k.into(),v)).collect(),..Default::default()}
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Sections {pub version:u8,pub sections:Vec<Section>}
impl Default for Sections {fn default()->Self {Self{version:1,sections:vec![]}}}
pub fn parse(raw:&str)->Result<Sections,String> {
    if raw.is_empty(){return Ok(Sections::default());}
    if raw.len()>262144{return Err("Page sections are too large to open.".into());}
    let doc:Sections=serde_json::from_str(raw).map_err(|_|"Page sections could not be read. Restore an earlier draft or contact the site administrator.")?;
    if doc.version!=1||doc.sections.len()>MAX_SECTIONS{return Err("Use up to eight approved sections.".into());}
    let mut ids=std::collections::HashSet::new();
    for s in &doc.sections {
        if s.id==0||!ids.insert(s.id)||!OPTIONS.iter().any(|(k,_,_)|*k==s.kind){return Err("Choose an approved section with a unique identifier.".into());}
    }
    Ok(doc)
}
pub fn validate(raw:&str)->Result<Sections,String> {
    if raw.len()>65000{return Err("Keep page sections under 65,000 characters.".into());}
    let doc=parse(raw)?;
    for s in &doc.sections {
        if s.title.len()>160||s.eyebrow.len()>80||s.body.len()>5000||s.alt.len()>500||s.caption.len()>600||s.credit.len()>240||s.button_text.len()>100||s.x>100||s.y>100{return Err(format!("{}: shorten the text or choose a focus from 0 to 100.",s.label()));}
        for value in [&s.image,&s.source_url,&s.button_url] {if value.len()>2048||!safe_url(value){return Err(format!("{}: use an HTTPS address or a path beginning with /.",s.label()));}}
    }
    Ok(doc)
}
pub fn safe_url(value:&str)->bool {
    value.is_empty()||(!value.chars().any(|c|c.is_control()||c.is_whitespace()||c=='\\')&&((value.starts_with('/')&&!value.starts_with("//"))||(value.starts_with("https://")&&value.len()>8&&!value[8..].starts_with('/'))))
}
pub fn issues(doc:&Sections)->Vec<String> {
    let mut issues=vec![];
    for (index,s) in doc.sections.iter().enumerate().filter(|(_,s)|!s.hidden) {
        let prefix=format!("Section {} ({})",index+1,s.label());
        if s.kind!="photo"&&s.title.trim().is_empty(){issues.push(format!("{prefix}: add a heading."));}
        if ["text","split"].contains(&s.kind.as_str())&&s.body.trim().is_empty(){issues.push(format!("{prefix}: add your text."));}
        if s.has_photo() {
            if s.image.trim().is_empty(){issues.push(format!("{prefix}: choose a photograph."));}
            else {for(value,label)in[(&s.alt,"alt text"),(&s.caption,"a caption"),(&s.credit,"a credit")]{if value.trim().is_empty(){issues.push(format!("{prefix}: add {label}."));}}}
        }
        if s.kind=="callout"||!s.button_text.is_empty()||!s.button_url.is_empty(){if s.button_text.trim().is_empty()||s.button_url.trim().is_empty(){issues.push(format!("{prefix}: add both button text and destination."));}}
    }
    issues
}
pub fn publishing_issues(d:&Draft)->Vec<String> {
    let mut result=vec![];
    if d.title.trim().is_empty(){result.push("Add a headline.".into());}
    if d.slug.trim().is_empty(){result.push("Add an address ending.".into());}
    if !d.field("image").is_empty()&&d.field("image_alt").trim().is_empty(){result.push("Add a description for the main photograph.".into());}
    match validate(&d.field(FIELD)){Ok(doc)=>result.extend(issues(&doc)),Err(error)=>result.push(error)}
    result
}

/// Keep hidden work in the staff record, out of anonymous HTML and JSON payloads.
#[cfg(feature="ssr")]
pub fn public_content(mut content:Content)->Content {
    if let Some(raw)=content.fields.get(FIELD) {
        match parse(raw) {
            Ok(mut doc) if doc.sections.iter().any(|s|s.hidden)=>{
                doc.sections.retain(|s|!s.hidden);
                content.fields.insert(FIELD.into(),serde_json::to_string(&doc).unwrap());
            }
            Err(_)=>{content.fields.remove(FIELD);}
            _=>{}
        }
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    fn encoded(sections:Vec<Section>)->String {serde_json::to_string(&Sections{version:1,sections}).unwrap()}
    #[test]
    fn incomplete_sections_can_save_but_cannot_publish_until_ready(){
        let mut photo=Section::new(1,"photo");assert_eq!(issues(&parse(&encoded(vec![photo.clone()])).unwrap()).len(),1);
        photo.image="/images/photo.webp".into();assert_eq!(issues(&parse(&encoded(vec![photo.clone()])).unwrap()).len(),3);
        photo.alt="A person clearing a trail".into();photo.caption="Trail maintenance, 2026".into();photo.credit="Photographer".into();assert!(issues(&parse(&encoded(vec![photo])).unwrap()).is_empty());
    }
    #[test]
    fn rejects_unapproved_shapes_unsafe_links_and_excessive_content(){
        for value in ["javascript:alert(1)","//other.example/image","https://example.test/\n","data:image/svg+xml,<svg>","/\\evil"]{let mut s=Section::new(1,"photo");s.image=value.into();assert!(validate(&encoded(vec![s])).is_err());}
        assert!(parse(&encoded(vec![Section::new(1,"html")])).is_err());assert!(parse(&encoded(vec![Section::new(1,"text");2])).is_err());assert!(parse(&encoded((1..=9).map(|id|Section::new(id,"text")).collect())).is_err());
        let mut s=Section::new(1,"text");s.body="x".repeat(5001);assert!(validate(&encoded(vec![s])).is_err());
    }
    #[test]
    fn ordering_and_hidden_drafts_round_trip_without_weakening_url_validation(){
        let mut hidden=Section::new(4,"photo");hidden.hidden=true;let d=parse(&encoded(vec![hidden.clone(),Section::new(2,"callout")])).unwrap();assert_eq!(d.sections.iter().map(|s|s.id).collect::<Vec<_>>(),vec![4,2]);assert_eq!(issues(&d).len(),2);
        hidden.image="javascript:bad".into();assert!(validate(&encoded(vec![hidden])).is_err());assert_eq!(parse("").unwrap(),Sections::default());
    }
    #[test]
    fn partial_link_edits_remain_open_in_the_editor(){
        let mut section=Section::new(1,"callout");section.button_url="htt".into();let raw=encoded(vec![section]);assert!(parse(&raw).is_ok());assert!(validate(&raw).is_err());
    }
    #[test]
    #[cfg(feature="ssr")]
    fn hidden_sections_are_removed_from_anonymous_payloads_without_changing_staff_data(){
        let mut hidden=Section::new(2,"text");hidden.hidden=true;hidden.body="Unfinished internal notes".into();
        let mut content=Content::default();content.fields.insert(FIELD.into(),encoded(vec![Section::new(1,"text"),hidden]));
        let public=public_content(content.clone());assert!(!public.field(FIELD).contains("Unfinished internal notes"));assert_eq!(parse(&public.field(FIELD)).unwrap().sections.len(),1);assert!(content.field(FIELD).contains("Unfinished internal notes"));
    }
}
