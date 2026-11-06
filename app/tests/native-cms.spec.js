const {test,expect}=require('@playwright/test');
const AxeBuilder=require('@axe-core/playwright').default;
const fs=require('fs');
const password=fs.readFileSync('.cms/rust/credentials.env','utf8').match(/^TRAIL_ADMIN_PASSWORD=(.*)$/m)[1].trim();
const origin='http://localhost:3110';
async function signIn(page){await page.goto('/admin');await page.getByLabel('Username',{exact:true}).fill('trail-admin');await page.getByLabel('Password',{exact:true}).fill(password);await page.getByRole('button',{name:'Sign in',exact:true}).click();await expect(page.getByRole('heading',{name:'Welcome back.'})).toBeVisible();}
async function session(request){const response=await request.post('/api/cms/login',{headers:{Origin:origin},data:{login:'trail-admin',password}});expect(response.ok()).toBeTruthy();return response.json();}
function headers(s){return {Origin:origin,'x-csrf-token':s.csrf};}
async function create(request,s,kind='story'){const r=await request.post('/api/cms/items',{headers:headers(s),data:{kind}});expect(r.ok()).toBeTruthy();return r.json();}
async function change(request,s,i,action,content,extra={}){const r=await request.post(`/api/cms/items/${i.id}`,{headers:headers(s),data:{action,version:i.version,content,...extra}});expect(r.ok(),await r.text()).toBeTruthy();return r.json();}
const draft=(title,slug)=>({title,slug,summary:'A story from the trail.',body:'A thoughtful update for visitors.',fields:{category:'Community'}});

test('Leptos editor autosaves, previews, publishes and restores a revision',async({page})=>{
 const errors=[];page.on('pageerror',e=>errors.push(e.message));await signIn(page);
 await page.getByRole('button',{name:'+ Write a story',exact:true}).click();await expect(page.getByRole('heading',{name:'A new beginning'})).toBeVisible();await expect(page).toHaveURL(/\/admin\/edit\/\d+$/);
 await page.getByLabel('Headline',{exact:true}).fill('Native editor story');await page.getByLabel('Address ending',{exact:true}).fill('native-editor-story');await page.getByLabel('Body text',{exact:true}).fill('A story saved from the Leptos editor.');await expect(page.locator('#save-status')).toHaveText('All changes saved.');
 await page.getByRole('button',{name:'Preview',exact:true}).click();await expect(page.getByRole('dialog')).toBeVisible();await expect(page.frameLocator('iframe').getByRole('heading',{name:'Native editor story'})).toBeVisible();await page.getByRole('button',{name:'Close preview'}).click();
 await page.getByRole('button',{name:'Publish changes',exact:true}).click();await expect(page.locator('#save-status')).toContainText('Published.');
 await page.getByLabel('Headline',{exact:true}).fill('A private second draft');await expect(page.locator('#save-status')).toHaveText('All changes saved.');
 const publicPage=await page.context().newPage();await publicPage.goto('/stories/native-editor-story');await expect(publicPage.getByRole('heading',{name:'Native editor story',exact:true})).toBeVisible();await publicPage.close();
 await page.getByRole('button',{name:'Publish changes',exact:true}).click();await expect(page.locator('#save-status')).toContainText('Published.');await page.getByRole('button',{name:'Restore as draft',exact:true}).first().click();await expect(page.getByLabel('Headline',{exact:true})).toHaveValue('Native editor story');await expect(page.locator('#save-status')).toContainText('Revision restored');
 expect(errors).toEqual([]);
});
test('API rejects anonymous writes, cross-origin login, missing CSRF and stale versions',async({request})=>{
 expect((await request.get('/api/cms/workspace')).status()).toBe(401);
 expect((await request.post('/api/cms/login',{headers:{Origin:'https://unrelated.example'},data:{login:'trail-admin',password}})).status()).toBe(403);
 const s=await session(request);expect((await request.post('/api/cms/items',{headers:{Origin:origin},data:{kind:'story'}})).status()).toBe(403);
 const i=await create(request,s);const d=draft('API story','api-story');await change(request,s,i,'save',d);
 expect((await request.post(`/api/cms/items/${i.id}`,{headers:headers(s),data:{action:'save',version:i.version,content:d}})).status()).toBe(409);
 const cookies=await request.storageState();expect(cookies.cookies[0].httpOnly).toBe(true);expect(cookies.cookies[0].sameSite).toBe('Strict');
});
test('document replacement keeps its stable download URL and rejects unsafe uploads',async({request})=>{
 const s=await session(request);const upload=await request.post('/api/cms/media',{headers:headers(s),multipart:{file:{name:'trail-guide.pdf',mimeType:'application/pdf',buffer:Buffer.from('%PDF-1.4\nNative test PDF\n%%EOF')}}});expect(upload.ok()).toBeTruthy();const m=await upload.json();
 let i=await create(request,s,'resource');let d=draft('Native guide','native-guide');d.fields={url:m.url,format:'PDF'};i=await change(request,s,i,'publish',d);
 let r=await request.get(`/api/cms/download/${i.id}`,{maxRedirects:0});expect(r.headers().location).toBe(m.url);d.fields.url='https://iditarod100.org/';i=await change(request,s,i,'publish',d);r=await request.get(`/api/cms/download/${i.id}`,{maxRedirects:0});expect(r.headers().location).toBe('https://iditarod100.org/');
 r=await request.post('/api/cms/media',{headers:headers(s),multipart:{file:{name:'evil.svg',mimeType:'image/svg+xml',buffer:Buffer.from('<svg onload="alert(1)"></svg>')}}});expect(r.status()).toBe(400);
 r=await request.get(m.url);expect(r.headers()['content-type']).toBe('application/pdf');expect(r.headers()['content-disposition']).toContain('attachment');
});
test('contributor can submit their own draft but cannot publish, inspect others, or manage people',async({request,playwright})=>{
 const s=await session(request);const result=await request.post('/api/cms/users',{headers:headers(s),data:{login:'test-writer',name:'Test Writer',role:'contributor',password:'test-writer-password-123'}});expect(result.ok()).toBeTruthy();
 const writer=await playwright.request.newContext({baseURL:origin});const login=await writer.post('/api/cms/login',{headers:{Origin:origin},data:{login:'test-writer',password:'test-writer-password-123'}});const w=await login.json();let i=await create(writer,w);i=await change(writer,w,i,'submit',draft('Contributor draft','contributor-draft'));expect(i.status).toBe('pending');
 expect((await writer.post(`/api/cms/items/${i.id}`,{headers:headers(w),data:{action:'publish',version:i.version,content:i.draft}})).status()).toBe(403);
 const workspace=await (await writer.get('/api/cms/workspace')).json();expect(workspace.items.every(i=>i.owner===w.user.id)).toBe(true);
 expect((await writer.post('/api/cms/users',{headers:headers(w),data:{login:'illegal',name:'No',role:'admin',password:'unsafe-privilege-123'}})).status()).toBe(403);await writer.dispose();
});
test('native dashboard, editor and login are accessible and responsive',async({page})=>{
 await page.goto('/admin');await expect(page.getByLabel('Username',{exact:true})).toBeVisible();let results=await new AxeBuilder({page}).analyze();expect(results.violations).toEqual([]);await signIn(page);
 for(const width of [1440,768,360]){await page.setViewportSize({width,height:1000});await expect.poll(()=>page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth+1)).toBe(true);await page.screenshot({path:`artifacts/native-studio-${width}.png`,fullPage:true});}
 await page.getByRole('button',{name:'Toggle workspace menu'}).click();await expect(page.locator('.studio-sidebar')).toHaveClass(/open/);await page.getByRole('button',{name:'Toggle workspace menu'}).click();
 await page.setViewportSize({width:1440,height:1000});results=await new AxeBuilder({page}).analyze();expect(results.violations).toEqual([]);
 await page.goto('/admin/page');await page.getByRole('link',{name:'Edit →'}).first().click();await expect(page.getByLabel('Headline',{exact:true})).toBeVisible();results=await new AxeBuilder({page}).analyze();expect(results.violations).toEqual([]);await page.screenshot({path:'artifacts/native-studio-editor.png',fullPage:true});
});
test('homepage choices use titles and file upload works from the Leptos form',async({page})=>{
 await signIn(page);await page.goto('/admin/page');await page.getByRole('link',{name:'A trail through Alaska.',exact:true}).click();await expect(page.getByLabel('Featured story',{exact:true})).toBeVisible();await page.getByLabel('Featured story',{exact:true}).selectOption({label:'A private second draft'});await page.locator('.place-choices input').first().check();await expect(page.locator('#save-status')).toHaveText('All changes saved.');await page.reload();await expect(page.getByLabel('Featured story',{exact:true})).toHaveValue(/\d+/);
 await page.goto('/admin/media');await page.getByLabel('Choose a photo or PDF',{exact:true}).setInputFiles({name:'from-studio.pdf',mimeType:'application/pdf',buffer:Buffer.from('%PDF-1.4\nUploaded from Leptos\n%%EOF')});await expect(page.getByRole('heading',{name:'from-studio.pdf'})).toBeVisible();
 const css=await page.request.get('/studio-native.css');expect(css.headers()['content-type']).toContain('text/css');
});
test('all staff sections load without browser errors and public HTML works without JavaScript',async({page,browser})=>{
 const errors=[];page.on('pageerror',e=>errors.push(e.message));await signIn(page);for(const route of ['page','story','place','resource','event','campaign','media','connections','team','help']){await page.goto(`/admin/${route}`);await expect(page.locator('#studio-main h1')).toBeVisible();expect(await page.locator('body').innerText()).not.toContain('WordPress');}expect(errors).toEqual([]);
 const context=await browser.newContext({javaScriptEnabled:false});const nojs=await context.newPage();await nojs.goto(origin+'/');await expect(nojs.locator('h1')).toContainText('A trail through Alaska.');await nojs.goto(origin+'/stories/native-editor-story');await expect(nojs.locator('h1')).toContainText('A private second draft');await context.close();
});
