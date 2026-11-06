const {test, expect} = require('@playwright/test');
const AxeBuilder = require('@axe-core/playwright').default;
const routeUrl = path => (process.env.TRAIL_BASE_PATH || '') + path;
const routes=['/','/explore','/places/seward','/places/chugach','/places/winter-trail','/stories','/stories/a-living-connection','/stories/the-winter-way','/stories/the-next-generation','/get-involved','/store','/about','/resources','/preview','/credits'];

for(const width of [360,768,1440]) {
 test(`All routes render without overflow or broken photos at ${width}px`,async({page})=>{
  await page.setViewportSize({width,height:900});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  for(const route of routes){
   const response=await page.goto(routeUrl(route));expect(response.status()).toBe(200);
   await expect(page.locator('h1')).toHaveCount(1);
   const audit=await page.evaluate(async()=>{
    const images=[...document.images];images.forEach(i=>i.loading='eager');await Promise.all(images.map(i=>i.decode().catch(()=>{})));
    return {overflow:document.documentElement.scrollWidth>window.innerWidth,broken:images.filter(i=>!i.naturalWidth).map(i=>i.src),rawBreak:document.body.innerText.includes('<br/>')};
   });
   expect(audit,route).toEqual({overflow:false,broken:[],rawBreak:false});
  }
  expect(errors).toEqual([]);
 });
}
test('Season switch and mobile navigation work by keyboard and touch',async({page})=>{
 await page.setViewportSize({width:390,height:844});await page.goto(routeUrl('/'));
 await page.getByRole('button',{name:'Summer',exact:true}).click();
 await expect(page.locator('.hero-summer')).toHaveCSS('opacity','1');
 await expect(page.locator('.hero-winter')).toHaveCSS('opacity','0');
 await page.getByRole('button',{name:'Open menu'}).click();await expect(page.locator('#main-nav')).toBeVisible();
 await page.keyboard.press('Escape');await expect(page.locator('#main-nav')).not.toBeVisible();
 await page.getByRole('button',{name:'Open menu'}).click();await page.locator('#main-nav').getByRole('link',{name:'Plan a trip'}).click();
 await expect(page).toHaveURL(/\/explore\/?$/);await expect(page.locator('#main-nav')).not.toBeVisible();
});
test('Landscape motion pauses while seasonal and chapter controls keep working',async({page})=>{
 await page.goto(routeUrl('/'));
 await expect.poll(()=>page.locator('.snowflake').first().evaluate(el=>el.getAnimations().some(a=>a.playState==='running'))).toBe(true);
 await page.getByRole('button',{name:'Pause motion',exact:true}).click();
 await expect(page.getByRole('button',{name:'Resume motion'})).toHaveAttribute('aria-pressed','true');
 await expect.poll(()=>page.locator('.motion-scene').evaluate(el=>el.getAnimations({subtree:true}).filter(a=>a.playState==='running').length)).toBe(0);
 await page.getByRole('button',{name:'Summer',exact:true}).click();
 await expect(page.locator('.hero-summer')).toHaveCSS('opacity','1');
 const chapters=page.getByRole('group',{name:'Trail chapters'});
 await chapters.getByRole('button',{name:'04 Nome'}).click();
 await expect(page.locator('#trail-chapter')).toContainText('The trail ends. The connection stays.');
 await expect(page.locator('.chapter-content')).toHaveCSS('opacity','1');
 await page.getByRole('button',{name:'Resume motion'}).click();
 await expect(page.getByRole('button',{name:'Pause motion',exact:true})).toHaveAttribute('aria-pressed','false');
});
test('Trail chapters support keyboard selection and retain focus as the story changes',async({page})=>{
 await page.goto(routeUrl('/'));
 const chapters=page.getByRole('group',{name:'Trail chapters'});
 const next=chapters.getByRole('button',{name:'02 Southcentral'});
 await next.focus();await page.keyboard.press('Enter');
 await expect(next).toBeFocused();await expect(next).toHaveAttribute('aria-pressed','true');
 await expect(page.locator('#trail-chapter')).toContainText('A little further into the wild.');
 await expect(page.getByRole('img',{name:/Selected chapter: Southcentral/})).toBeVisible();
 await page.keyboard.press('Tab');await page.keyboard.press('Space');
 await expect(chapters.getByRole('button',{name:'03 Interior'})).toBeFocused();
 await expect(page.locator('#trail-chapter')).toContainText('Carried forward, generation to generation.');
 await page.locator('#trail-chapter').getByRole('link',{name:'Meet the living trail'}).click();
 await expect(page).toHaveURL(/\/stories\/a-living-connection\/?$/);
 await page.getByRole('link',{name:'Iditarod Historic Trail Alliance home'}).first().click();
 await chapters.getByRole('button',{name:'04 Nome'}).click();
 await expect(page.locator('#trail-chapter')).toContainText('The trail ends. The connection stays.');
});
test('System reduced-motion preference keeps content visible and interaction immediate',async({page})=>{
 await page.emulateMedia({reducedMotion:'reduce'});await page.goto(routeUrl('/'));
 await expect(page.getByText('Reduced motion',{exact:true})).toBeVisible();
 await expect(page.getByRole('button',{name:'Pause motion',exact:true})).toHaveCount(0);
 await expect.poll(()=>page.locator('.motion-scene').evaluate(el=>el.getAnimations({subtree:true}).filter(a=>a.playState==='running').length)).toBe(0);
 await expect(page.locator('.snowfall')).not.toBeVisible();
 await page.getByRole('button',{name:'Summer',exact:true}).click();await expect(page.locator('.hero-summer')).toHaveCSS('opacity','1');
 await page.getByRole('button',{name:'03 Interior'}).click();await expect(page.locator('.chapter-content')).toHaveCSS('opacity','1');
 await expect(page.locator('#trail-chapter')).toContainText('Alaska Native travel networks');
});
test('Server-rendered homepage remains readable without JavaScript',async({browser})=>{
 const context=await browser.newContext({javaScriptEnabled:false});const page=await context.newPage();
 await page.goto((process.env.DEMO_URL||'http://127.0.0.1:3107')+routeUrl('/'));
 await expect(page.locator('h1')).toBeVisible();
 await expect(page.getByRole('heading',{name:'One trail. Countless ways to belong.'})).toBeVisible();
 await expect(page.locator('#trail-chapter')).toContainText('Where the mountains meet the sea.');
 await expect(page.locator('.journey-grid').locator('..')).toHaveCSS('opacity','1');
 await context.close();
});
test('Trip filters combine and recover from empty results',async({page})=>{
 await page.goto(routeUrl('/explore'));await expect(page.locator('.places-results .place-card')).toHaveCount(3);
 await page.getByRole('button',{name:'Summer',exact:true}).click();await expect(page.locator('.places-results .place-card')).toHaveCount(2);
 await page.getByRole('combobox',{name:'Region'}).selectOption('Across the trail');await expect(page.locator('.places-results .place-card')).toHaveCount(0);
 await page.getByRole('button',{name:'Clear filters'}).click();await expect(page.locator('.places-results .place-card')).toHaveCount(3);
 await page.getByRole('searchbox',{name:'Search places'}).fill('Seward');await expect(page.locator('.places-results .place-card')).toHaveCount(1);
 await page.locator('.places-results .place-card').click();await expect(page).toHaveURL(/\/places\/seward\/?$/);
});
test('Timeline and resource search respond to selections',async({page})=>{
 await page.goto(routeUrl('/stories/a-living-connection'));await page.getByRole('button',{name:'03 1978'}).click();await expect(page.locator('.timeline-content')).toContainText('Congress designates');
 await page.goto(routeUrl('/resources'));await page.getByRole('searchbox',{name:'Search resources'}).fill('classroom');await expect(page.locator('.resource-row')).toHaveCount(1);
 await page.getByRole('button',{name:'History',exact:true}).click();await expect(page.locator('.resource-row')).toHaveCount(0);await page.getByRole('button',{name:'Clear search'}).click();await expect(page.locator('.resource-row')).toHaveCount(7);
});
test('Gift and membership previews validate amounts without making requests',async({page})=>{
 const posts=[];page.on('request',r=>{if(r.method()==='POST')posts.push(r.url())});
 await page.goto(routeUrl('/get-involved'));await page.getByRole('button',{name:'$100',exact:true}).click();await page.getByRole('button',{name:'Monthly',exact:true}).click();await page.getByRole('button',{name:'Preview gift · $100'}).click();await expect(page.locator('.confirmation')).toContainText('$100 contribution per month');
 await page.getByRole('button',{name:'Explore another option'}).click();await page.getByRole('button',{name:'Become a member',exact:true}).click();await page.getByRole('button',{name:'Preview membership · $35'}).click();await expect(page.locator('.confirmation')).toContainText('$35 membership');expect(posts).toEqual([]);
});
test('Bag quantities, route persistence, totals, and demo checkout',async({page})=>{
 await page.goto(routeUrl('/store'));const add=page.getByRole('button',{name:'Add to bag'});await add.nth(0).click();await add.nth(1).click();await expect(page.locator('.bag-total')).toContainText('$30');
 await page.getByRole('button',{name:'Add one The trail field journal'}).click();await expect(page.locator('.bag-total')).toContainText('$48');
 await page.locator('#main-nav').getByRole('link',{name:'Learn & discover'}).click();await page.getByRole('link',{name:'Trail shop, 3 items in bag'}).click();await expect(page.locator('.bag-total')).toContainText('$48');
 await page.getByRole('button',{name:'Remove one The living trail patch'}).click();await expect(page.locator('.bag-total')).toContainText('$36');await page.getByRole('button',{name:'Try the demo checkout'}).click();await expect(page.locator('.checkout-confirmation')).toContainText('Preview subtotal: $36');await expect(page.locator('.empty-bag')).toBeVisible();
});
test('Editor preview updates locally',async({page})=>{
 await page.goto(routeUrl('/preview'));await page.getByRole('textbox',{name:'Story headline'}).fill('A new chapter for Alaska');await expect(page.locator('.editor-preview h3')).toHaveText('A new chapter for Alaska');await page.getByRole('combobox',{name:'Featured photograph'}).selectOption('summer');await expect(page.locator('.editor-preview img')).toHaveAttribute('src',routeUrl('/images/summer-small.webp'));await page.getByRole('button',{name:'Save demo preview'}).click();await expect(page.locator('.editor-controls [role=status]')).toContainText('public site has not changed');
});
for(const path of ['/','/explore','/get-involved','/store','/resources','/preview']){
 test(`Accessibility checks: ${path}`,async({page})=>{
  await page.goto(routeUrl(path));await page.evaluate(()=>document.fonts.ready);
  // Inspect each revealed section at its final readable contrast, as a visitor sees it.
  for(const section of await page.locator('.motion-reveal').all()){
   await section.scrollIntoViewIfNeeded();await expect(section).toHaveCSS('opacity','1');
   for(const card of await section.locator('.experience-card,.place-card').all())await expect(card).toHaveCSS('opacity','1');
  }
  await page.evaluate(()=>window.scrollTo({top:0,behavior:'instant'}));
  const {violations}=await new AxeBuilder({page}).withTags(['wcag2a','wcag2aa','wcag21aa','wcag22aa']).analyze();
  expect(violations.map(v=>({id:v.id,impact:v.impact,nodes:v.nodes.map(n=>n.target)}))).toEqual([]);
 });
}
