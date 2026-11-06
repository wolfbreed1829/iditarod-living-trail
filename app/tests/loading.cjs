const {chromium,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('node:fs'),path=require('node:path');
(async()=>{
 const origin=process.env.EDITORIAL_URL||'http://localhost:3115';
 const out=process.env.EDITORIAL_ARTIFACT_DIR||'artifacts/loading';fs.mkdirSync(out,{recursive:true});
 const b=await chromium.launch({channel:'chrome',headless:true});const checks=[],audits=[],metrics=[],errors=[];
 try {
  const c=await b.newContext({baseURL:origin,viewport:{width:390,height:844},deviceScaleFactor:1,reducedMotion:'reduce'});
  const p=await c.newPage();p.setDefaultTimeout(30000);p.on('pageerror',e=>errors.push(e.message));
  let release;const held=new Promise(resolve=>release=resolve);
  await p.route('**/*.wasm',async route=>{await held;await route.continue();});
  await p.goto('/',{waitUntil:'domcontentloaded'});
  await expect(p.locator('h1')).toContainText('A trail');
  await expect(p.locator('#runtime-status')).toBeVisible();
  await expect(p.locator('.season-switch button').first()).toBeDisabled();
  await p.keyboard.press('Tab');await expect(p.getByRole('link',{name:'Skip to content',exact:true})).toBeFocused();
  await p.keyboard.press('Enter');await expect(p.locator('#main')).toBeFocused();
  await p.locator('.mobile-navigation summary').focus();await p.keyboard.press('Enter');
  await expect(p.locator('#mobile-nav')).toBeVisible();await p.keyboard.press('Tab');
  await expect(p.locator('#mobile-nav a').first()).toBeFocused();await p.keyboard.press('Escape');
  await expect(p.locator('#mobile-nav')).not.toBeVisible();await expect(p.locator('.mobile-navigation summary')).toBeFocused();
  await p.screenshot({path:path.join(out,'loading-mobile.png')});
  audits.push({state:'loading',violations:(await new AxeBuilder({page:p}).analyze()).violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>n.target)}))});
  checks.push('SSR content, skip link, native menu and Escape work while WASM is delayed; enhanced controls wait');
  release();await expect(p.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});
  await expect(p.locator('#runtime-status')).not.toBeVisible();await expect(p.locator('.season-switch button').first()).toBeEnabled();
  expect(await p.locator('.hero-landscape img').count()).toBe(1);
  expect(await p.evaluate(()=>performance.getEntriesByType('resource').filter(r=>/\/images\/summer(?:-small)?\.webp/.test(r.name)).length)).toBe(0);
  await p.locator('.season-switch').getByRole('button',{name:'Summer',exact:true}).focus();await p.keyboard.press('Enter');
  await expect(p.locator('.hero-summer')).toHaveCount(1);await expect(p.locator('.explorer-experience h4')).toContainText('Summer');
  const motion=await p.evaluate(()=>({animations:document.getAnimations().filter(a=>a.playState==='running').length,scroll:getComputedStyle(document.documentElement).scrollBehavior}));
  expect(motion).toEqual({animations:0,scroll:'auto'});checks.push('One seasonal hero, keyboard season selection and reduced motion');
  await p.screenshot({path:path.join(out,'ready-mobile.png')});
  audits.push({state:'ready',violations:(await new AxeBuilder({page:p}).analyze()).violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>n.target)}))});
  await p.unroute('**/*.wasm');
  await p.route('**/*.wasm',route=>route.abort('failed'));
  await p.goto('/support/volunteer',{waitUntil:'domcontentloaded'});
  await expect(p.locator('#runtime-status')).toContainText('could not load');await expect(p.getByRole('button',{name:'Try again',exact:true})).toBeEnabled();
  await p.unroute('**/*.wasm');await p.getByRole('button',{name:'Try again',exact:true}).click();
  await expect(p.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});await expect(p.locator('#runtime-status')).not.toBeVisible();
  checks.push('Failed download has truthful feedback and a working retry');await c.close();
  const nojs=await b.newContext({baseURL:origin,javaScriptEnabled:false,viewport:{width:390,height:844}}),n=await nojs.newPage();
  await n.goto('/');await expect(n.locator('h1')).toBeVisible();await n.locator('.mobile-navigation summary').click();await n.locator('#mobile-nav a[href="/about"]').click();await expect(n).toHaveURL(/\/about$/);await expect(n.locator('h1')).toBeVisible();await nojs.close();checks.push('Pages and mobile navigation work without JavaScript');
  if(process.env.PERF_NETWORK==='1'){
   const net=await b.newContext({baseURL:origin,viewport:{width:390,height:844},deviceScaleFactor:2,reducedMotion:'reduce'}),q=await net.newPage();
   const client=await net.newCDPSession(q);await client.send('Network.enable');
   await client.send('Network.emulateNetworkConditions',{offline:false,latency:150,downloadThroughput:125000,uploadThroughput:62500});
   for(const route of ['/','/explore','/admin','/']){
    await q.goto(route,{waitUntil:'domcontentloaded',timeout:90000});await expect(q.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});
    await q.waitForTimeout(500);const data=await q.evaluate(()=>({resources:performance.getEntriesByType('resource').map(r=>({name:new URL(r.name).pathname,transfer:r.transferSize,encoded:r.encodedBodySize,decoded:r.decodedBodySize,ms:r.duration})),paint:performance.getEntriesByName('first-contentful-paint')[0]?.startTime}));
    const wasm=data.resources.filter(r=>r.name.endsWith('.wasm'));expect(wasm).toHaveLength(1);expect(wasm[0].name).toMatch(/living-trail\.[a-f0-9]{16}\.wasm$/);
    if(!metrics.length){expect(wasm[0].encoded).toBeLessThanOrEqual(1100*1024);expect(data.resources.reduce((s,r)=>s+r.transfer,0)).toBeLessThan(1800*1024);expect(data.resources.filter(r=>r.name.endsWith('.webp')).reduce((s,r)=>s+r.encoded,0)).toBeLessThan(750*1024);expect(data.paint).toBeLessThan(5500);}
    else expect(wasm[0].transfer).toBe(0);
    metrics.push({route,...data});
   }
   const wasm=metrics[0].resources.find(r=>r.name.endsWith('.wasm'));
   const response=await q.request.get(wasm.name,{headers:{'Accept-Encoding':'br'}});expect(response.headers()['cache-control']).toContain('immutable');expect(response.headers()['content-encoding']).toBe('br');
   expect((await q.request.get('/admin')).headers()['cache-control']).toBe('no-store');expect((await q.request.get('/api/cms/workspace')).headers()['cache-control']).toBe('no-store');
   await net.close();checks.push('1 Mbps / 150 ms mobile budget; repeat WASM transfers zero; private documents remain uncached');
  }
  expect(errors).toEqual([]);expect(audits.flatMap(a=>a.violations)).toEqual([]);
  fs.writeFileSync(path.join(out,'loading.json'),JSON.stringify({origin,checks,audits,metrics,errors},null,2));console.log(JSON.stringify({checks,metrics:metrics.map(m=>({route:m.route,total:m.resources.reduce((s,r)=>s+r.transfer,0),wasm:m.resources.find(r=>r.name.endsWith('.wasm')),fcp:m.paint}))},null,2));
 }finally{await b.close();}
})().catch(e=>{console.error(e.stack);process.exit(1)});
