const {chromium,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('fs');
(async()=>{
 const origin=process.env.EDITORIAL_URL||'http://localhost:3110';
 const b=await chromium.launch({channel:'chrome',headless:true});
 const c=await b.newContext({baseURL:origin,viewport:{width:1440,height:1000},reducedMotion:'reduce'});
 const p=await c.newPage();p.setDefaultTimeout(30000);p.setDefaultNavigationTimeout(90000);
 const out=process.env.EDITORIAL_ARTIFACT_DIR||'artifacts/editorial';fs.mkdirSync(out,{recursive:true});const artifact=name=>require('path').join(out,name);const errors=[];p.on('pageerror',e=>errors.push(e.message));const checks=[],axe=[];
 for(const width of [1440,768,390]){await p.setViewportSize({width,height:1000});for(const path of ['/','/explore','/places/seward','/stories/a-living-connection','/stories','/about','/resources','/get-involved','/store','/preview','/credits']){
  const r=await p.goto(path);expect(r.status()).toBe(200);await expect(p.locator('h1')).toHaveCount(1);
  await expect(p.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});
  const audit=await p.evaluate(async()=>{for(const i of document.images)i.loading='eager';await Promise.all([...document.images].map(i=>i.decode().catch(()=>{})));return{overflow:document.documentElement.scrollWidth>innerWidth+1,broken:[...document.images].filter(i=>!i.naturalWidth).map(i=>i.src)}});
  expect(audit,`${width} ${path}`).toEqual({overflow:false,broken:[]});checks.push({width,path,passed:true});
  if(width===1440){const result=await new AxeBuilder({page:p}).analyze();axe.push({path,violations:result.violations.map(v=>({id:v.id,impact:v.impact,nodes:v.nodes.map(n=>n.target)}))});}
  if(width!==768)await p.screenshot({path:artifact(`pass2-${path.slice(1).replaceAll('/','-')||'home'}-${width}.png`),fullPage:true});
 }}
 await p.goto('/resources');await p.getByRole('searchbox',{name:'Search resources',exact:true}).fill('no-such-resource-8202');await expect(p.getByRole('button',{name:'Clear search'})).toBeVisible();await p.getByRole('button',{name:'Clear search'}).click();await expect(p.locator('.resource-row').first()).toBeVisible();
 await p.goto('/store');await expect(p.getByRole('button',{name:/checkout|Add to bag/})).toHaveCount(0);await expect(p.locator('input[type=email]')).toHaveCount(0);
 await p.setViewportSize({width:1440,height:1000});await p.goto('/');await expect(p.locator('html')).toHaveAttribute('data-hydrated','true');await p.locator('.explorer-pin[data-place="nome"]').click();await expect(p.locator('#trail-chapter h3')).toHaveText('Nome.');await p.locator('.season-switch').getByRole('button',{name:'Summer',exact:true}).click();await expect(p.locator('.hero-summer')).toHaveCSS('opacity','1');await p.setViewportSize({width:390,height:844});await p.locator('.mobile-navigation summary').click();await expect(p.locator('#mobile-nav')).toBeVisible();await p.keyboard.press('Escape');await expect(p.locator('#mobile-nav')).not.toBeVisible();
 const report={origin,checks,axe,errors,atlas:true,season:true,mobile_menu:true};fs.writeFileSync(artifact(`editorial-public-${origin.includes('localhost')?'local':'azure'}.json`),JSON.stringify(report,null,2));expect(errors).toEqual([]);expect(axe.flatMap(a=>a.violations)).toEqual([]);console.log(`Pass 2: ${checks.length} responsive route checks; ${axe.length} accessibility audits; atlas, seasonal and mobile navigation interactions.`);await b.close();
})().catch(e=>{console.error(e.message);process.exit(1)});
