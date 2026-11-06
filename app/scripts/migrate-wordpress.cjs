// One-time migration. Run before retiring the old containers; never overwrites an existing DB.
const fs = require('fs');
const {execFileSync} = require('child_process');
const php = `<?php require '/var/www/html/wp-load.php';
$items=[]; foreach(get_posts(['post_type'=>array_map(fn($t)=>'trail_'.$t,TRAIL_TYPES),'post_status'=>['publish','draft','pending','private'],'numberposts'=>-1]) as $p) {
 $r=trail_record($p,true); $r['owner']=(int)$p->post_author; $r['ever_published']=!!get_post_meta($p->ID,'_trail_ever_published',true)||$p->post_status==='publish'; $items[]=$r;
}
$users=[]; foreach(get_users() as $u) $users[]=['id'=>(int)$u->ID,'login'=>$u->user_login,'name'=>$u->display_name,'role'=>in_array('administrator',$u->roles)?'admin':(in_array('editor',$u->roles)?'editor':'contributor')];
echo json_encode(['items'=>$items,'users'=>$users,'connections'=>(object)get_option('trail_connections',[])]);`;
const data=execFileSync('docker',['exec','-i','iditarod-cms-wordpress-1','php'],{input:php,encoding:'utf8'});
const parsed=JSON.parse(data);
if(parsed.users.length!==1) throw new Error('Multiple staff accounts require an explicit password migration. Original accounts are preserved.');
fs.mkdirSync('.cms/rust',{recursive:true});
fs.writeFileSync('.cms/rust/import.json',JSON.stringify(parsed,null,2));
const env=fs.readFileSync('.cms/local.env','utf8');
const password=env.match(/^TRAIL_ADMIN_PASSWORD=(.*)$/m)[1].trim();
if(!fs.existsSync('.cms/rust/credentials.env')) fs.writeFileSync('.cms/rust/credentials.env',`TRAIL_ADMIN_USER=${parsed.users[0].login}\nTRAIL_ADMIN_PASSWORD=${password}\n`);
fs.mkdirSync('.cms/rust/legacy-uploads',{recursive:true});
execFileSync('docker',['cp','iditarod-cms-wordpress-1:/var/www/html/wp-content/uploads/.','.cms/rust/legacy-uploads']);
console.log(`Exported ${parsed.items.length} records and ${parsed.users.length} account; uploads and original volumes preserved.`);
