const {chromium,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('fs'),path=require('path');
(async()=>{
 const origin=process.env.EDITORIAL_URL||'http://localhost:3115';
 const out=process.env.EDITORIAL_ARTIFACT_DIR||'artifacts/interiors';fs.mkdirSync(out,{recursive:true});
 const b=await chromium.launch({channel:'chrome',headless:true});const c=await b.newContext({baseURL:origin,reducedMotion:'reduce'});const p=await c.newPage();p.setDefaultTimeout(30000);p.setDefaultNavigationTimeout(90000);
 const errors=[],checks=[],audits=[];p.on('pageerror',e=>errors.push(e.message));
 const routes=['/places/seward','/places/chugach','/places/winter-trail','/stories/a-living-connection','/stories/the-winter-way','/stories/the-next-generation','/about','/stories','/resources','/get-involved','/store','/credits'];
 const photos=new Set();
 for(const width of [1440,768,390,360]){
  await p.setViewportSize({width,height:1000});
  for(const route of routes){
   expect((await p.goto(route)).status()).toBe(200);await expect(p.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});await expect(p.locator('h1')).toHaveCount(1);
   const layout=await p.evaluate(async()=>{for(const i of document.images)i.loading='eager';await Promise.all([...document.images].map(i=>i.decode().catch(()=>{})));return{overflow:document.documentElement.scrollWidth>innerWidth+1,broken:[...document.images].filter(i=>!i.naturalWidth).map(i=>i.src)}});expect(layout,`${route} at ${width}`).toEqual({overflow:false,broken:[]});
   if(route.startsWith('/places/')){await expect(p.locator('.destination-experience-grid article')).toHaveCount(3);await expect(p.locator('.destination-photo-sequence figure')).toHaveCount(2);await expect(p.locator('.planning-notes>div')).toHaveCount(3);await expect(p.locator('.supporting-map')).toHaveCount(1);}
   if(route.startsWith('/stories/')){await expect(p.locator('.magazine-chapter')).toHaveCount(3);await expect(p.locator('.magazine-sequence-photo')).toHaveCount(2);await expect(p.locator('.supporting-map')).toHaveCount(1);}
   if(route==='/about'){await expect(p.locator('.alliance-timeline article')).toHaveCount(5);await expect(p.locator('.preservation-work-grid article')).toHaveCount(3);}
   if(width===1440){if(route!=='/credits'){const image=p.locator('.interior-hero-photo>img,.support-landscape>img,.resource-prelude-photo>img,.shop-opening>figure>img').first();if(await image.count())photos.add(await image.getAttribute('src'));}const a=await new AxeBuilder({page:p}).analyze();audits.push({route,width,violations:a.violations.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))});}
   if(width===390&&['/places/seward','/stories/the-winter-way','/about','/credits'].includes(route)){const a=await new AxeBuilder({page:p}).analyze();audits.push({route,width,violations:a.violations.map(v=>({id:v.id,targets:v.nodes.map(n=>n.target)}))});}
   if((width===1440||width===390)&&['/places/seward','/places/chugach','/stories/a-living-connection','/stories/the-winter-way','/about','/credits'].includes(route))await p.screenshot({path:path.join(out,route.slice(1).replaceAll('/','-')+'-'+width+'.png'),fullPage:true});
   checks.push({route,width});
  }
 }
 await p.goto('/credits');await expect(p.locator('.credit-photo')).toHaveCount(35);await p.getByLabel('Photograph category',{exact:true}).selectOption('Stewardship');await expect(p.locator('.credit-photo')).toHaveCount(3);await p.getByLabel('Search the collection',{exact:true}).fill('tripod');await expect(p.locator('.credit-photo')).toHaveCount(1);await expect(p.locator('.credit-photo')).toContainText('Kevin Keeler');await p.getByLabel('Search the collection',{exact:true}).fill('no-such-photo-123');await p.getByRole('button',{name:'Show all photographs'}).click();await expect(p.locator('.credit-photo')).toHaveCount(35);
 await p.goto('/places/seward');await p.locator('.supporting-map-action').click();await expect(p).toHaveURL(/explore\?place=seward/);await expect(p.locator('#trail-chapter h3')).toHaveText('Seward.');
 const nojs=await b.newContext({javaScriptEnabled:false});const staticPage=await nojs.newPage();await staticPage.goto(origin+'/stories/the-winter-way');await expect(staticPage.locator('.magazine-sequence-photo')).toHaveCount(2);await expect(staticPage.locator('.archive-label').first()).toBeVisible();await nojs.close();
 const report={origin,checks,audits,errors,collection_size:35,filters:true,guide_to_explorer:true,nojs:true,primary_photos_detected:photos.size};fs.writeFileSync(path.join(out,'interiors-'+(origin.includes('localhost')?'local':'azure')+'.json'),JSON.stringify(report,null,2));expect(photos.size).toBe(11);expect(errors).toEqual([]);expect(audits.flatMap(a=>a.violations)).toEqual([]);console.log(`${checks.length} responsive route checks and ${audits.length} accessibility audits passed; library filters, map links and server-rendered stories verified.`);await b.close();
})().catch(e=>{console.error(e.stack);process.exit(1)});
