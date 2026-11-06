const {chromium,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('fs'),path=require('path');
(async()=>{
 const origin=process.env.EDITORIAL_URL||'http://localhost:3115';
 if(!['localhost','127.0.0.1'].includes(new URL(origin).hostname)||process.env.EDITORIAL_WRITE_TESTS!=='1')throw Error('Use an isolated local server and EDITORIAL_WRITE_TESTS=1.');
 if(!process.env.TRAIL_TEST_PASSWORD)throw Error('Set TRAIL_TEST_PASSWORD.');
 const out=process.env.EDITORIAL_ARTIFACT_DIR||'artifacts/explorer';fs.mkdirSync(out,{recursive:true});
 const b=await chromium.launch({channel:'chrome',headless:true});const c=await b.newContext({baseURL:origin,viewport:{width:1440,height:1000},reducedMotion:'reduce'});const p=await c.newPage();p.setDefaultTimeout(30000);const errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto('/admin');await p.getByLabel('Username',{exact:true}).fill(process.env.TRAIL_TEST_LOGIN||'trail-admin');await p.getByLabel('Password',{exact:true}).fill(process.env.TRAIL_TEST_PASSWORD);await p.getByRole('button',{name:'Sign in',exact:true}).click();await expect(p.getByRole('heading',{name:'Welcome back.'})).toBeVisible();
 const session=await(await p.request.get('/api/cms/session')).json();const headers={Origin:origin,'x-csrf-token':session.csrf};
 await p.goto('/admin/place');await p.getByRole('button',{name:'+ Add new',exact:true}).click();await expect(p.getByLabel('Headline',{exact:true})).toBeVisible();const id=Number(p.url().split('/').at(-1));
 try{
  await p.getByLabel('Headline',{exact:true}).fill('Knik explorer workflow');await p.getByLabel('Address ending',{exact:true}).fill('explorer-workflow-'+Date.now());await p.getByLabel('Short introduction',{exact:true}).fill('An introduction reviewed in Trail Studio.');
  await p.getByLabel('Place on the map',{exact:true}).selectOption('knik');await p.getByLabel('Explorer history',{exact:true}).fill('A reviewed history prepared for the map.');await p.getByLabel('History source address',{exact:true}).fill('https://www.iditarod100.org/knik.html');await p.getByLabel('Summer ideas',{exact:true}).fill('Townsite history\nHistoric photography');await p.getByLabel('Winter ideas',{exact:true}).fill('Explore winter heritage');await p.getByLabel('Photo caption',{exact:true}).fill('Knik townsite, September 2014.');await p.getByLabel('Visitor information address',{exact:true}).fill('https://www.iditarod100.org/knik.html');
  expect(await p.locator('.curated-photo').count()).toBe(35);await p.locator('.curated-photo').filter({has:p.locator('img[src="/images/explorer/knik-townsite-cabin-small.webp"]')}).click();await expect(p.getByLabel('Photo credit',{exact:true})).toHaveValue('Cherime MacFarlane');await p.getByLabel('Photo caption',{exact:true}).fill('Knik townsite, September 2014.');
  await expect(p.locator('#save-status')).toHaveText('All changes saved.');await p.reload();await expect(p.getByLabel('Place on the map',{exact:true})).toHaveValue('knik');await expect(p.getByLabel('Summer ideas',{exact:true})).toHaveValue('Townsite history\nHistoric photography');
  const audit=await new AxeBuilder({page:p}).analyze();expect(audit.violations.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))).toEqual([]);
  await p.setViewportSize({width:390,height:844});expect(await p.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);await p.getByLabel('Place on the map',{exact:true}).scrollIntoViewIfNeeded();await p.screenshot({path:path.join(out,'explorer-cms-mobile.png')});await p.setViewportSize({width:1440,height:1000});
  const publicPage=await c.newPage();await publicPage.goto('/explore?place=knik&season=summer');await expect(publicPage.locator('.explorer-history p')).not.toHaveText('A reviewed history prepared for the map.');
  await p.getByRole('button',{name:'Preview',exact:true}).click();await expect(p.frameLocator('iframe').locator('h1')).toHaveText('Knik explorer workflow');await p.getByRole('button',{name:'Close preview'}).click();
  await p.getByRole('button',{name:'Publish changes',exact:true}).click();await expect(p.locator('#save-status')).toContainText('Published.');await publicPage.reload();await expect(publicPage.locator('.explorer-history p')).toHaveText('A reviewed history prepared for the map.');await expect(publicPage.locator('.explorer-place-photo figcaption')).toContainText('Knik townsite, September 2014.');await expect(publicPage.locator('.explorer-place-photo img')).toHaveAttribute('src','/images/explorer/knik-townsite-cabin.webp');await expect(publicPage.locator('.explorer-experience')).toContainText('Townsite history');
  await p.getByLabel('Explorer history',{exact:true}).fill('A later private revision.');await expect(p.locator('#save-status')).toHaveText('All changes saved.');await publicPage.reload();await expect(publicPage.locator('.explorer-history p')).toHaveText('A reviewed history prepared for the map.');
  fs.writeFileSync(path.join(out,'explorer-cms.json'),JSON.stringify({origin,photo_count:35,editor_accessibility_violations:[],mobile_overflow:false,fields_persist:true,private_preview:true,published_override:true,later_draft_private:true,errors},null,2));expect(errors).toEqual([]);
 }finally{
  const workspace=await(await p.request.get('/api/cms/workspace')).json();const item=workspace.items.find(i=>i.id===id);const r=await p.request.post('/api/cms/items/'+id,{headers,data:{action:'archive',version:item.version}});expect(r.ok(),await r.text()).toBe(true);
 }
 console.log('Explorer CMS: autosave, 35 photo choices, responsive editor, accessibility, private preview, published map override and draft privacy passed.');await b.close();
})().catch(e=>{console.error(e.stack);process.exit(1)});
