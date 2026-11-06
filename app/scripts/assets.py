from pathlib import Path
from io import BytesIO
import requests, re, json
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'public' / 'images'
OUT.mkdir(parents=True, exist_ok=True)
SOURCES = [
 ('dawn','https://live.staticflickr.com/4346/36599186731_ff88a7e6b0_k.jpg','https://www.flickr.com/photos/mypubliclands/36599186731/','Bob Wick / Bureau of Land Management','CC BY 2.0'),
 ('dogs','https://live.staticflickr.com/4389/36738421625_e2acbc9a53_k.jpg','https://www.flickr.com/photos/mypubliclands/36738421625/','Bob Wick / Bureau of Land Management','CC BY 2.0'),
 ('cycling','https://live.staticflickr.com/4406/36568558672_432efc2302_k.jpg','https://www.flickr.com/photos/mypubliclands/36568558672/','Bob Wick / Bureau of Land Management','CC BY 2.0'),
 ('summer','https://live.staticflickr.com/8663/29533359703_0c704ff9ce_b.jpg','https://www.flickr.com/photos/alaska_region/29533359703/','USDA Forest Service Alaska Region','CC BY 2.0'),
 ('waterfront','https://www.nps.gov/kefj/blogs/images/Rez-bay-440.jpg','https://www.nps.gov/kefj/blogs/one-fjord-to-another.htm','NPS / Max Odland','Public domain'),
 ('winter','https://www.blm.gov/sites/default/files/hero_backgrounds/Programs_NLCS_IditarodTrail_Hero.jpg','https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod','Bureau of Land Management','US government source'),
]
manifest=[]
for name,url,source,credit,license in SOURCES:
 response=requests.get(url,timeout=30)
 if response.status_code!=200 and '_k.jpg' in url:
  response=requests.get(url.replace('_k.jpg','_b.jpg'),timeout=30)
 response.raise_for_status()
 photo=Image.open(BytesIO(response.content)).convert('RGB')
 photo.thumbnail((2200,2200));photo.save(OUT/(name+'.webp'),'WEBP',quality=86)
 photo.thumbnail((800,800));photo.save(OUT/(name+'-small.webp'),'WEBP',quality=82)
 manifest.append(dict(name=name,source=source,credit=credit,license=license,changes='Resized, WebP conversion, responsive display crops'))
 print('Downloaded',name,flush=True)
(ROOT/'public'/'image-sources.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
fonts=ROOT/'public'/'fonts';fonts.mkdir(exist_ok=True)
headers={'User-Agent':'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36'}
for family,name,folder in [('DM+Sans:wght@400..700','dm-sans','dmsans'),('Fraunces:opsz,wght@9..144,400..600','fraunces','fraunces')]:
 response=requests.get('https://fonts.googleapis.com/css2?family='+family+'&display=swap',headers=headers,timeout=30);response.raise_for_status()
 urls=re.findall(r'url\((https://[^)]+)\)',response.text)
 if not urls:raise RuntimeError('Font source not found: '+name)
 response=requests.get(urls[-1],timeout=30);response.raise_for_status();(fonts/(name+'.woff2')).write_bytes(response.content)
 response=requests.get('https://raw.githubusercontent.com/google/fonts/main/ofl/'+folder+'/OFL.txt',timeout=30);response.raise_for_status();(fonts/(folder+'-OFL.txt')).write_text(response.text,encoding='utf-8')
