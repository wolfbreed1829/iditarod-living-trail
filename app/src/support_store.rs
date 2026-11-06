//! Durable, private support enquiries. This store never records a payment or activates membership.
use crate::{studio_model::User, studio_store::{Store, Result, digest, now}, support_model::*};
use rusqlite::{params, Connection, OptionalExtension};
use chrono::Utc;
fn db_error(e:rusqlite::Error)->String {eprintln!("Support database: {e}");"Could not save the request. Please try again.".into()}
pub fn init(db:&Connection)->Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS support_requests(id INTEGER PRIMARY KEY, token TEXT NOT NULL UNIQUE, fingerprint TEXT NOT NULL, data TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS support_limits(client TEXT PRIMARY KEY, bucket INTEGER NOT NULL, count INTEGER NOT NULL);").map_err(db_error)
}
fn valid_key(key:&str)->bool {key.len()==48&&key.bytes().all(|b|b.is_ascii_hexdigit())}
fn clean(mut s:Submission)->Result<Submission> {
    if !valid_key(&s.key) || !KINDS.iter().any(|(kind,_,_)|*kind==s.kind) {return Err("Choose a valid way to help and reload the form.".into());}
    if !s.website.is_empty() {return Err("The request could not be accepted. Please leave the website field empty.".into());}
    s.name=s.name.trim().into();s.email=s.email.trim().to_lowercase();s.message=s.message.trim().into();s.availability=s.availability.trim().into();s.amount=s.amount.trim().into();
    if s.name.len()>160||s.message.len()>2000||s.availability.len()>240||s.email.len()>254||s.email.chars().any(|c|c.is_whitespace()||c.is_control()) {return Err("Check the length and format of your details.".into());}
    let parts:Vec<_>=s.email.split('@').collect();
    if parts.len()!=2 || parts[0].is_empty() || !parts[1].contains('.') || parts[1].starts_with('.') || parts[1].ends_with('.') {return Err("Enter a valid email address.".into());}
    if s.kind!="newsletter"&&s.name.is_empty() {return Err("Add your name so the team can follow up.".into());}
    if !s.consent {return Err("Please confirm how the team may use your details.".into());}
    if !AREAS.iter().any(|(area,_)|*area==s.area) {return Err("Choose an area of interest.".into());}
    if !s.amount.is_empty() && (s.kind!="giving"||!s.amount.parse::<u32>().is_ok_and(|n|(1..=1_000_000).contains(&n))) {return Err("Use a whole-dollar amount from 1 to 1,000,000, or leave it blank.".into());}
    if s.kind=="newsletter" {s.name.clear();s.amount.clear();s.message.clear();s.availability.clear();s.area="where-needed".into();}
    if s.kind!="volunteer" {s.availability.clear();}
    Ok(s)
}
fn receipt(r:&SupportRecord)->Receipt {Receipt{reference:r.reference.clone(),kind:r.kind.clone(),area:r.area.clone(),created:r.created.clone(),withdrawn:r.status=="withdrawn"}}
fn read_record(db:&Connection,id:u64)->Result<SupportRecord> {
    let data:Option<String>=db.query_row("SELECT data FROM support_requests WHERE id=?",[id as i64],|r|r.get(0)).optional().map_err(db_error)?;
    serde_json::from_str(&data.ok_or("Not found.")?).map_err(|_|"Could not read the request.".into())
}
fn write(db:&Connection,r:&SupportRecord)->Result<()> {db.execute("UPDATE support_requests SET data=? WHERE id=?",params![serde_json::to_string(r).unwrap(),r.id as i64]).map_err(db_error)?;Ok(())}
fn authorize(u:&User)->Result<()> {if u.publisher(){Ok(())}else{Err("Only editors and administrators can read support requests.".into())}}
impl Store {
    pub fn support_submit(&self,input:Submission,client:&str)->Result<Receipt> {
        let s=clean(input)?;let key=digest(&s.key);let fingerprint=digest(&serde_json::to_string(&s).unwrap());
        let mut db=self.db.lock().unwrap();let tx=db.transaction().map_err(db_error)?;
        let existing:Option<(String,String)>=tx.query_row("SELECT fingerprint,data FROM support_requests WHERE token=?",[&key],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(db_error)?;
        if let Some((saved,data))=existing {if saved!=fingerprint{return Err("This form was already submitted. Open a new form for a different request.".into());}let r:SupportRecord=serde_json::from_str(&data).map_err(|_|"Could not read the request.")?;return Ok(receipt(&r));}
        let bucket=Utc::now().timestamp()/3600;let client=digest(&format!("support:{client}"));
        tx.execute("DELETE FROM support_limits WHERE bucket<?",[bucket-1]).map_err(db_error)?;
        let count:i64=tx.query_row("SELECT count FROM support_limits WHERE client=? AND bucket=?",params![client,bucket],|r|r.get(0)).optional().map_err(db_error)?.unwrap_or(0);
        if count>=10 {return Err("Too many requests. Please try again in an hour.".into());}
        tx.execute("INSERT INTO support_limits(client,bucket,count) VALUES(?,?,1) ON CONFLICT(client) DO UPDATE SET bucket=excluded.bucket,count=CASE WHEN support_limits.bucket=excluded.bucket THEN support_limits.count+1 ELSE 1 END",params![client,bucket]).map_err(db_error)?;
        let stamp=now();let mut r=SupportRecord{id:0,reference:format!("TRAIL-{}",s.key[..10].to_uppercase()),kind:s.kind,name:s.name,email:s.email,area:s.area,amount:s.amount,availability:s.availability,message:s.message,consent_version:CONSENT_VERSION.into(),created:stamp.clone(),updated:stamp,status:"new".into(),note:String::new(),version:1};
        tx.execute("INSERT INTO support_requests(token,fingerprint,data) VALUES(?,?,'')",params![key,fingerprint]).map_err(db_error)?;r.id=tx.last_insert_rowid() as u64;write(&tx,&r)?;tx.commit().map_err(db_error)?;Ok(receipt(&r))
    }
    pub fn support_receipt(&self,key:&str)->Result<Receipt> {
        if !valid_key(key){return Err("Not found.".into());}let db=self.db.lock().unwrap();let id:Option<u64>=db.query_row("SELECT id FROM support_requests WHERE token=?",[digest(key)],|r|r.get::<_,i64>(0).map(|v|v as u64)).optional().map_err(db_error)?;Ok(receipt(&read_record(&db,id.ok_or("Not found.")?)?))
    }
    pub fn support_withdraw(&self,key:&str)->Result<Receipt> {
        if !valid_key(key){return Err("Not found.".into());}let db=self.db.lock().unwrap();let id:Option<u64>=db.query_row("SELECT id FROM support_requests WHERE token=?",[digest(key)],|r|r.get::<_,i64>(0).map(|v|v as u64)).optional().map_err(db_error)?;let mut r=read_record(&db,id.ok_or("Not found.")?)?;
        if r.status!="withdrawn" {r.status="withdrawn".into();r.name.clear();r.email.clear();r.message.clear();r.availability.clear();r.note.clear();r.amount.clear();r.updated=now();r.version+=1;write(&db,&r)?;}Ok(receipt(&r))
    }
    pub fn support_list(&self,u:&User)->Result<Vec<SupportRecord>> {
        authorize(u)?;let db=self.db.lock().unwrap();let mut stmt=db.prepare("SELECT data FROM support_requests ORDER BY id DESC").map_err(db_error)?;
        let rows=stmt.query_map([],|r|r.get::<_,String>(0)).map_err(db_error)?;rows.map(|r|serde_json::from_str(&r.map_err(db_error)?).map_err(|_|"Could not read support requests.".into())).collect()
    }
    pub fn support_change(&self,u:&User,id:u64,c:SupportChange)->Result<()> {
        authorize(u)?;let db=self.db.lock().unwrap();let mut r=read_record(&db,id)?;
        if r.version!=c.version{return Err("Conflict: this request changed. Reload before updating it.".into());}
        if c.delete {if u.role!="admin"{return Err("Only administrators can delete support requests.".into());}db.execute("DELETE FROM support_requests WHERE id=?",[id as i64]).map_err(db_error)?;return Ok(());}
        if r.status=="withdrawn" {return Err("A withdrawn request cannot be reopened.".into());}
        if !["new","in-progress","closed"].contains(&c.status.as_str())||c.note.len()>2000{return Err("Choose a valid status and keep the note under 2,000 characters.".into());}
        r.status=c.status;r.note=c.note.trim().into();r.version+=1;r.updated=now();write(&db,&r)
    }
    pub fn support_csv(&self,u:&User,kind:&str)->Result<String> {
        if u.role!="admin" {return Err("Only administrators can export contact details.".into());}
        if !KINDS.iter().any(|(k,_,_)|*k==kind){return Err("Choose a valid request type.".into());}
        fn cell(s:&str)->String {let prefix=if s.trim_start().starts_with(['=','+','-','@','\t','\r','\n']){"'"}else{""};format!("\"{}{}\"",prefix,s.replace('"',"\"\""))}
        let mut csv="reference,name,email,type,interest,amount_usd,availability,message,status,consent_version,requested_at\r\n".to_string();
        for r in self.support_list(u)?.into_iter().filter(|r|r.kind==kind&&r.status!="withdrawn") {csv.push_str(&[r.reference,r.name,r.email,r.kind,r.area,r.amount,r.availability,r.message,r.status,r.consent_version,r.created].iter().map(|s|cell(s)).collect::<Vec<_>>().join(","));csv.push_str("\r\n");}Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup()->Store {
        let dir=std::env::temp_dir().join(format!("support-tests-{}",crate::studio_store::token()));std::fs::create_dir_all(&dir).unwrap();let db=Connection::open(dir.join("trail.sqlite3")).unwrap();init(&db).unwrap();
        Store{db:std::sync::Mutex::new(db),dir,origin:"http://localhost:3115".into(),attempts:std::sync::Mutex::new(std::collections::HashMap::new())}
    }
    fn user(role:&str)->User {User{id:1,login:"reviewer".into(),name:"Reviewer".into(),role:role.into()}}
    fn submission(kind:&str)->Submission {Submission{key:crate::studio_store::token(),kind:kind.into(),name:"Trail Friend".into(),email:"friend@example.test".into(),area:"trail-care".into(),consent:true,..Default::default()}}
    #[test]
    fn requests_persist_and_retries_are_idempotent_without_public_contact_details(){
        let s=setup();let d=submission("giving");let receipt=s.support_submit(d.clone(),"client").unwrap();assert_eq!(s.support_submit(d.clone(),"client").unwrap(),receipt);assert_eq!(s.support_list(&user("admin")).unwrap().len(),1);
        let raw=serde_json::to_string(&s.support_receipt(&d.key).unwrap()).unwrap();assert!(!raw.contains("friend@example.test")&&!raw.contains("Trail Friend"));
        let other=Connection::open(s.dir.join("trail.sqlite3")).unwrap();assert_eq!(other.query_row("SELECT count(*) FROM support_requests",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        let mut changed=d;changed.message="Changed after an uncertain network response".into();assert!(s.support_submit(changed,"client").unwrap_err().contains("already submitted"));
    }
    #[test]
    fn invalid_input_and_missing_consent_never_create_records(){
        let s=setup();for modify in 0..6 {let mut d=submission("giving");match modify{0=>d.consent=false,1=>d.email="bad-address".into(),2=>d.kind="paid".into(),3=>d.amount="-50".into(),4=>d.website="spam".into(),_=>d.area="unknown".into()};assert!(s.support_submit(d,"client").is_err());}assert!(s.support_list(&user("admin")).unwrap().is_empty());
    }
    #[test]
    fn withdrawal_erases_contact_details_and_cannot_be_reopened_or_exported(){
        let s=setup();let d=submission("newsletter");let receipt=s.support_submit(d.clone(),"client").unwrap();assert!(!receipt.withdrawn);assert!(s.support_withdraw(&d.key).unwrap().withdrawn);
        let r=s.support_list(&user("admin")).unwrap().remove(0);assert!(r.name.is_empty()&&r.email.is_empty()&&r.message.is_empty());assert!(s.support_submit(d,"client").unwrap().withdrawn);assert!(!s.support_csv(&user("admin"),"newsletter").unwrap().contains("friend@example.test"));
        assert!(s.support_change(&user("admin"),r.id,SupportChange{version:r.version,status:"new".into(),note:String::new(),delete:false}).is_err());
    }
    #[test]
    fn permissions_conflicts_deletion_and_csv_formula_safety(){
        let s=setup();let mut d=submission("volunteer");d.name="=HYPERLINK(1)".into();s.support_submit(d.clone(),"client").unwrap();assert!(s.support_list(&user("contributor")).is_err());assert!(s.support_csv(&user("editor"),"volunteer").is_err());
        assert!(s.support_csv(&user("admin"),"volunteer").unwrap().contains("'=HYPERLINK"));let r=s.support_list(&user("editor")).unwrap().remove(0);
        let change=SupportChange{version:r.version,status:"in-progress".into(),note:"Reviewing logistics".into(),delete:false};s.support_change(&user("editor"),r.id,change.clone()).unwrap();assert!(s.support_change(&user("editor"),r.id,change).unwrap_err().starts_with("Conflict:"));
        let delete=SupportChange{version:2,status:"new".into(),note:String::new(),delete:true};assert!(s.support_change(&user("editor"),r.id,delete.clone()).is_err());s.support_change(&user("admin"),r.id,delete).unwrap();assert!(s.support_receipt(&d.key).is_err());
    }
    #[test]
    fn rate_limit_counts_new_submissions_but_allows_a_safe_retry(){
        let s=setup();let first=submission("membership");s.support_submit(first.clone(),"client").unwrap();for _ in 0..9{s.support_submit(submission("giving"),"client").unwrap();}assert!(s.support_submit(submission("giving"),"client").unwrap_err().contains("Too many"));assert!(s.support_submit(first,"client").is_ok());assert!(s.support_submit(submission("giving"),"another-client").is_ok());
    }
}
