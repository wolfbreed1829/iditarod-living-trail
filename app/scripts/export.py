"""Export fully rendered Leptos pages with their WASM hydration assets."""
import argparse
from pathlib import Path
import shutil
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
ROUTES = ['', 'explore', 'places/seward', 'places/chugach', 'places/winter-trail',
          'stories', 'stories/a-living-connection', 'stories/the-winter-way',
          'stories/the-next-generation', 'get-involved', 'store', 'about',
          'resources', 'preview', 'credits']
parser = argparse.ArgumentParser()
parser.add_argument('--origin', default='http://localhost:3109')
parser.add_argument('--base', default='/iditarod-living-trail')
args = parser.parse_args()
out = ROOT / 'dist'
out.mkdir(exist_ok=True)
for folder in ['pkg', 'images', 'fonts']:
    shutil.copytree(ROOT / 'target' / 'site' / folder, out / folder, dirs_exist_ok=True)
for name in ['favicon.svg', 'image-sources.json', 'cms.css']:
    shutil.copy2(ROOT / 'public' / name, out / name)
for route in ROUTES:
    address = args.origin + args.base + ('/' + route if route else '')
    with urllib.request.urlopen(address, timeout=20) as response:
        html = response.read()
    if b'<h1' not in html or b'<html' not in html:
        raise RuntimeError(f'Page did not render: {address}')
    dest = out / route / 'index.html'
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(html)
    print(route or '/', len(html), flush=True)
# An exported not-found view preserves meaningful navigation on static hosting.
try:
    with urllib.request.urlopen(args.origin + args.base + '/not-a-trail', timeout=20) as response:
        fallback = response.read()
except urllib.error.HTTPError as error:
    fallback = error.read()
(out / '404.html').write_bytes(fallback)
(out / '.nojekyll').write_text('')
(out / 'robots.txt').write_text('User-agent: *\nDisallow: /\n')
print('Export ready:', out)
