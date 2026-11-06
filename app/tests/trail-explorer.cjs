const {chromium,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('fs'),path=require('path');

(async()=>{
  const origin=process.env.EDITORIAL_URL||'http://localhost:3115';
  const out=process.env.EDITORIAL_ARTIFACT_DIR||'artifacts/explorer';fs.mkdirSync(out,{recursive:true});
  const browser=await chromium.launch({channel:'chrome',headless:true});
  const context=await browser.newContext({baseURL:origin,viewport:{width:1440,height:1100},reducedMotion:'reduce'});
  const page=await context.newPage();
  page.setDefaultTimeout(30000);page.setDefaultNavigationTimeout(90000);
  const errors=[],checks=[],accessibility=[];page.on('pageerror',e=>errors.push(e.message));
  const ready=async()=>expect(page.locator('html')).toHaveAttribute('data-hydrated','true',{timeout:90000});
  const season=label=>page.locator('.explorer-toolbar').getByRole('button',{name:label,exact:true});
  const name=()=>page.locator('#explorer-place-title');
  const audit=async(label)=>{const r=await new AxeBuilder({page}).include('.trail-explorer').analyze();accessibility.push({label,violations:r.violations.map(v=>({id:v.id,nodes:v.nodes.map(n=>({target:n.target,summary:n.failureSummary}))}))});};
  await page.goto('/');await ready();
  await page.getByRole('group',{name:'Landscape season'}).getByRole('button',{name:'Summer',exact:true}).click();
  await expect(season('Summer')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('.explorer-experience h4')).toContainText('Summer');
  await page.locator('.explorer-pin[data-place="nome"]').click();await expect(name()).toHaveText('Nome.');
  await season('Winter').click();await expect(page.locator('.hero-summer')).toHaveCSS('opacity','0');
  checks.push('Hero and explorer season controls share state');
  const places=['Seward','Crow Pass','Knik','Iditarod','Unalakleet','Nome'];
  const photos=[];
  for(const place of places){
    await page.locator(`.explorer-place-option[data-place="${place.toLowerCase().replaceAll(' ','-')}"]`).click();await expect(name()).toHaveText(`${place}.`);
    await expect(page.locator('.explorer-history p')).not.toBeEmpty();
    const src=await page.locator('.explorer-place-photo img').getAttribute('src');photos.push(src);
    await expect(page.locator('.explorer-place-photo img')).toHaveJSProperty('complete',true);
    expect(await page.locator('.explorer-place-photo img').evaluate(i=>i.naturalWidth)).toBeGreaterThan(0);
    const winter=await page.locator('.explorer-experience').innerText();await season('Summer').click();
    expect(await page.locator('.explorer-experience').innerText()).not.toEqual(winter);
    await expect(page.locator('.explorer-useful-links a').last()).toHaveAttribute('href',/season=summer/);
    await season('Winter').click();
    checks.push(`${place}: photograph, history, distinct seasonal guidance, useful links`);
  }
  expect(new Set(photos).size).toBe(6);
  await page.getByRole('button',{name:'Whole trail',exact:true}).click();
  await page.locator('.explorer-cluster').click();
  await expect(page.locator('.explorer-geography')).toHaveAttribute('viewBox','590 420 240 186.6667');
  await page.locator('.explorer-pin[data-place="crow-pass"]').click();await expect(name()).toHaveText('Crow Pass.');
  await expect(page.locator('.explorer-experience')).toContainText('avalanche');
  await page.getByRole('button',{name:'Next place',exact:true}).click();await expect(name()).toHaveText('Knik.');
  await page.getByRole('button',{name:'Previous place',exact:true}).click();await expect(name()).toHaveText('Crow Pass.');
  checks.push('Regional map reveals accurate nearby pins; previous/next follow the route');
  await audit('Desktop regional winter');
  await season('Summer').click();await page.getByRole('link',{name:'Link to this place',exact:true}).click();await ready();
  await expect(page).toHaveURL(/\/explore\?place=crow-pass&season=summer#trail-atlas/);
  await page.reload();await ready();await expect(name()).toHaveText('Crow Pass.');await expect(season('Summer')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('.explorer-more-places h2')).toHaveText('More summer experiences.');
  await season('Winter').click();await expect(page.locator('.explorer-more-places h2')).toHaveText('More winter experiences.');
  checks.push('Shareable place/season link survives refresh; season also filters published experiences');
  await page.goto('/explore');await ready();await page.locator('.explorer-pin[data-place="nome"]').focus();await page.keyboard.press('Enter');await expect(name()).toHaveText('Nome.');
  await audit('Desktop whole trail');
  await page.locator('.explorer-shell').screenshot({path:path.join(out,'explorer-desktop.png')});
  checks.push('Map is keyboard operable');
  for(const width of [768,390,360]){
    await page.setViewportSize({width,height:844});await page.goto('/explore?place=seward&season=summer');await ready();
    await expect(page.getByRole('group',{name:'Places along the trail'})).toBeVisible();await expect(page.locator('.explorer-map-panel')).not.toBeVisible();
    await page.locator('.explorer-place-option[data-place="unalakleet"]').click();await expect(name()).toHaveText('Unalakleet.');await expect(page.locator('#trail-chapter')).toBeFocused();
    await page.getByRole('button',{name:'Map',exact:true}).click();await expect(page.locator('.explorer-map-panel')).toBeVisible();
    await expect(page.getByRole('group',{name:'Places along the trail'})).not.toBeVisible();
    const dimensions=await page.locator('.explorer-map').boundingBox();expect(Math.abs(dimensions.width/dimensions.height-9/7)).toBeLessThan(.01);
    await page.locator('.explorer-cluster').click();
    await page.locator('.explorer-pin[data-place="knik"]').click();await expect(name()).toHaveText('Knik.');
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth+1)).toBe(true);
    await page.getByRole('button',{name:/^List/,exact:false}).click();
    await audit(`Mobile list ${width}`);
    checks.push(`${width}px: list/map switching, regional pins, detail focus and no horizontal overflow`);
    if(width===390){await page.locator('.explorer-shell').screenshot({path:path.join(out,'explorer-mobile-list.png')});await page.getByRole('button',{name:'Map',exact:true}).click();await page.locator('.explorer-map-panel').screenshot({path:path.join(out,'explorer-mobile-map.png')});await audit('Mobile regional map');}
  }
  const report={origin,checks,accessibility,errors};fs.writeFileSync(path.join(out,'trail-explorer.json'),JSON.stringify(report,null,2));
  expect(errors).toEqual([]);expect(accessibility.flatMap(a=>a.violations)).toEqual([]);
  console.log(`${checks.length} explorer checks; ${accessibility.length} accessibility audits passed.`);await browser.close();
})().catch(e=>{console.error(e.stack);process.exit(1)});
