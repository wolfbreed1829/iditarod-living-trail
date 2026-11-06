"""Build the paper-atlas explorer base without changing earlier map assets.

Requires Python 3 and requests. Network inputs are pinned to a Natural Earth
commit. Run beside trail-orientation.svg and map-sources.json. All coastline
and historic-route SVG path strings are copied unchanged from the prior map.
"""
from pathlib import Path
import concurrent.futures
import hashlib
import html
import json
import math
import xml.etree.ElementTree as ET
import requests

HERE = Path(__file__).resolve().parent
SOURCE_SVG = HERE / 'trail-orientation.svg'
SOURCE_NOTES = HERE / 'map-sources.json'
SOURCE_HASHES = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (SOURCE_SVG, SOURCE_NOTES)}
COMMIT = '0e1681ff63fbc58c19dafed9e5b06733512b370d'
ROOT = f'https://raw.githubusercontent.com/nvkelso/natural-earth-vector/{COMMIT}/geojson/'
RIVERS_URL = ROOT + 'ne_50m_rivers_lake_centerlines.geojson'
MARINE_URL = ROOT + 'ne_10m_geography_marine_polys.geojson'

def download(url):
    response = requests.get(url, timeout=60)
    response.raise_for_status()
    return response.json()

with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
    rivers_future = pool.submit(download, RIVERS_URL)
    marine_future = pool.submit(download, MARINE_URL)
    rivers, marine = rivers_future.result(), marine_future.result()

notes = json.loads(SOURCE_NOTES.read_text(encoding='utf-8'))
view = notes['views']['trail-orientation']
assert view['viewBox'] == '0 0 900 700'
transform = view['transform']
rad = math.radians
n = (math.sin(rad(55)) + math.sin(rad(65))) / 2
c = math.cos(rad(55)) ** 2 + 2 * n * math.sin(rad(55))
rho0 = math.sqrt(c - 2 * n * math.sin(rad(50))) / n

def xy(lon, lat):
    delta = (lon + 154 + 180) % 360 - 180
    rho = math.sqrt(c - 2 * n * math.sin(rad(lat))) / n
    theta = n * rad(delta)
    px = 6371008.8 * rho * math.sin(theta)
    py = 6371008.8 * (rho0 - rho * math.cos(theta))
    return [px * transform['scale'] + transform['xOffset'], -py * transform['scale'] + transform['yOffset']]

def svg_path(coordinates):
    return 'M' + 'L'.join(f'{x:.2f},{y:.2f}' for x, y in (xy(*point) for point in coordinates))

source = ET.parse(SOURCE_SVG).getroot()
ns = {'s': 'http://www.w3.org/2000/svg'}
source_land = source.find('.//s:g[@class="map-land"]', ns)
land_paths = [path.attrib['d'] for path in source_land]
route_path = source.find('.//s:path[@class="map-route"]', ns).attrib['d']
land = ''.join(f'<path d="{d}"/>' for d in land_paths)

# A geographic graticule: meridians every 5 degrees and parallels every 2.
# Projected with the exact same transform as the original map and pins.
grid_paths = []
for lon in range(-175, -134, 5):
    grid_paths.append(svg_path([[lon, 55], [lon, 73]]))
for lat in range(56, 73, 2):
    grid_paths.append(svg_path([[-176 + step * .5, lat] for step in range(87)]))
grid = ''.join(f'<path d="{d}"/>' for d in grid_paths)

river_names = {'Copper', 'Koyukuk', 'Kuskokwim', 'N. Fork Kuskokwim', 'Porcupine', 'Susitna', 'Tanana', 'Yukon'}
selected_rivers = [f for f in rivers['features'] if f['properties'].get('name') in river_names]
river_elements = []
river_manifest = []
for river in selected_rivers:
    name = river['properties']['name']
    geometry = river['geometry']
    lines = [geometry['coordinates']] if geometry['type'] == 'LineString' else geometry['coordinates']
    river_manifest.append({'name': name, 'featureClass': river['properties']['featurecla'], 'sourceScaleRank': river['properties']['scalerank'], 'vertices': sum(len(line) for line in lines)})
    for line in lines:
        width = 1.45 if name == 'Yukon' else 1.0
        river_elements.append(f'<path class="explorer-river" data-river="{html.escape(name)}" d="{svg_path(line)}" stroke-width="{width}"/>')

def in_ring(point, ring):
    x, y = point
    inside = False
    previous = ring[-1]
    for current in ring:
        x1, y1 = previous
        x2, y2 = current
        if (y1 > y) != (y2 > y):
            if x < (x2 - x1) * (y - y1) / (y2 - y1) + x1:
                inside = not inside
        previous = current
    return inside

def in_geometry(point, geometry):
    polygons = [geometry['coordinates']] if geometry['type'] == 'Polygon' else geometry['coordinates']
    return any(in_ring(point, polygon[0]) and not any(in_ring(point, hole) for hole in polygon[1:]) for polygon in polygons)

# Label anchors lie within their Natural Earth marine polygons. They are
# cartographic placements inside the visible sector, not claimed centroids.
water_labels = [
    {'text': 'Bering Sea', 'longitude': -167.4, 'latitude': 61.8, 'size': 19, 'rotation': -8},
    {'text': 'Norton Sound', 'longitude': -163.3, 'latitude': 63.95, 'size': 15, 'rotation': 0},
    {'text': 'Gulf of Alaska', 'longitude': -146.9, 'latitude': 59.15, 'size': 18, 'rotation': 0},
]
water_elements = []
for label in water_labels:
    feature = next(f for f in marine['features'] if f['properties']['name'] == label['text'])
    assert in_geometry([label['longitude'], label['latitude']], feature['geometry']), label['text']
    x, y = xy(label['longitude'], label['latitude'])
    label.update({'x': round(x, 2), 'y': round(y, 2), 'anchorWithinSourceMarinePolygon': True})
    water_elements.append(f'<text class="explorer-water-label" x="{x:.2f}" y="{y:.2f}" text-anchor="middle" font-size="{label["size"]}" transform="rotate({label["rotation"]} {x:.2f} {y:.2f})">{label["text"]}</text>')

# Alaska is identified by the same Natural Earth state polygon as the coast.
# The Alaska Range anchor is USGS GNIS feature 1397815's first official point;
# lettering is intentionally a name only, not fabricated contours or relief.
land_labels = [
    {'text': 'ALASKA', 'longitude': -153.4, 'latitude': 65.9, 'size': 18, 'spacing': 4.5, 'rotation': 0, 'anchor': 'middle', 'source': 'Natural Earth Alaska state polygon'},
    {'text': 'Alaska Range', 'longitude': -153.083357, 'latitude': 62.583334, 'size': 14, 'spacing': 1.0, 'rotation': -22, 'anchor': 'start', 'source': 'USGS GNIS 1397815, first official multipoint position'},
]
land_elements = []
for label in land_labels:
    x, y = xy(label['longitude'], label['latitude'])
    label.update({'x': round(x, 2), 'y': round(y, 2)})
    land_elements.append(f'<text class="explorer-land-label" x="{x:.2f}" y="{y:.2f}" text-anchor="{label["anchor"]}" font-size="{label["size"]}" letter-spacing="{label["spacing"]}" transform="rotate({label["rotation"]} {x:.2f} {y:.2f})">{label["text"]}</text>')

svg = f'''<svg xmlns="http://www.w3.org/2000/svg" class="explorer-map-base" viewBox="0 0 900 700" fill="none" role="img" aria-labelledby="explorer-base-title explorer-base-description">
  <title id="explorer-base-title">The Iditarod historic trail across Alaska</title>
  <desc id="explorer-base-description">A geographic overview showing Alaska's coastline, major rivers and the BLM historic trail alignment from Seward to Nome. It excludes connecting historic trails and current race courses. For orientation, not navigation.</desc>
  <defs>
    <clipPath id="explorer-base-viewport"><rect width="900" height="700"/></clipPath>
    <clipPath id="explorer-base-land-clip">{land}</clipPath>
  </defs>
  <g clip-path="url(#explorer-base-viewport)">
    <rect class="explorer-water" width="900" height="700" fill="#dce6e1"/>
    <g class="explorer-land" fill="#f1ecdf" stroke="#9dab9d" stroke-width="1.05" stroke-linejoin="round" fill-rule="evenodd">{land}</g>
    <g class="explorer-grid" stroke="#617e71" stroke-width="0.55" stroke-opacity="0.18" fill="none">{grid}</g>
    <g clip-path="url(#explorer-base-land-clip)" stroke="#91b3b0" stroke-opacity="0.72" fill="none" stroke-linecap="round" stroke-linejoin="round">{''.join(river_elements)}</g>
    <g fill="#64796f" font-family="Georgia, serif" text-anchor="middle" letter-spacing="1.25" font-style="italic">{''.join(water_elements)}</g>
    <g fill="#7a806e" font-family="Georgia, serif" text-anchor="middle" opacity="0.75">{''.join(land_elements)}</g>
    <path class="explorer-route-casing" d="{route_path}" stroke="#fffaf0" stroke-opacity="0.94" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <path class="explorer-route" d="{route_path}" stroke="#a7653e" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/>
  </g>
</svg>'''
(HERE / 'explorer-base.svg').write_text(svg, encoding='utf-8')

manifest = {
    'retrieved': '2026-11-06',
    'file': 'explorer-base.svg',
    'viewBox': '0 0 900 700',
    'purpose': 'Decorative geographic base for the interactive six-place explorer. Site code owns all place pins and interactive labels.',
    'sourceGeometry': {'coastline': 'Exact unchanged path strings from trail-orientation.svg', 'historicRoute': 'Exact unchanged path string from trail-orientation.svg', 'sourceFileSha256': SOURCE_HASHES},
    'projection': view['projection'],
    'transform': transform,
    'pinCoordinates': 'Use map-sources.json views.trail-orientation.pins without changes.',
    'styling': {'method': 'All colors are SVG presentation attributes, which application CSS class rules can override. No inline style attributes or external resources.', 'classes': ['explorer-map-base','explorer-land','explorer-water','explorer-river','explorer-route','explorer-route-casing','explorer-grid','explorer-water-label','explorer-land-label']},
    'graticule': {'meridianIntervalDegrees':5,'parallelIntervalDegrees':2,'projection':'Same Albers transform as map and pins'},
    'rivers': river_manifest,
    'waterLabels': water_labels,
    'landLabels': land_labels,
    'notes': [
        'No place pins, artificial terrain, hill shading, contours or current conditions are drawn.',
        'Marine label anchors were verified inside the corresponding Natural Earth marine polygons; typography is cartographic placement, not a centroid assertion.',
        'River geometry is projected from Natural Earth without added bends or invented channels; clipped to the existing coast polygon to avoid coastline/generalization mismatch.',
        'Alaska Range is a named context label at its first USGS GNIS position. Chugach labels are omitted to preserve clear space around Crow Pass and Knik.',
        'The historic route is an overview, not a current race course or guarantee of access. Connecting historic trails are excluded by the source dataset.'
    ],
    'recommendedVisibleCredit':'Historic route: BLM / NPS. Geography: Natural Earth, USGS GNIS. For orientation only; not a navigation map.',
    'sources': [
        {'title':'Existing verified coast, BLM/NPS historic route, projection and GNIS place coordinates','localFile':'map-sources.json','terms':'See original full credits and BLM supplemental information in map-sources.json.'},
        {'title':'Natural Earth rivers and lake centerlines, 1:50m','url':RIVERS_URL,'documentation':'https://www.naturalearthdata.com/downloads/50m-physical-vectors/50m-rivers-lake-centerlines/','license':'Public domain','terms':'https://www.naturalearthdata.com/about/terms-of-use/','commit':COMMIT},
        {'title':'Natural Earth marine area polygons, 1:10m','url':MARINE_URL,'license':'Public domain','use':'Water-label anchor validation; polygon outlines are not rendered.','commit':COMMIT},
        {'title':'USGS GNIS Alaska Range, feature 1397815','url':'https://carto.nationalmap.gov/arcgis/rest/services/geonames/MapServer/5/query?where=gaz_id%3D1397815&outFields=gaz_id,gaz_name,gaz_featureclass&outSR=4326&f=pjson','license':'US public-domain geographic names data','coordinate':[-153.08335727340304,62.58333431792868]}
    ]
}
(HERE / 'explorer-map-sources.json').write_text(json.dumps(manifest, indent=2, ensure_ascii=False)+'\n', encoding='utf-8')

# Validate exact source path preservation and prevent accidental source changes.
output = ET.parse(HERE / 'explorer-base.svg').getroot()
new_land = output.find('.//s:g[@class="explorer-land"]', ns)
assert [p.attrib['d'] for p in new_land] == land_paths
assert output.find('.//s:path[@class="explorer-route"]', ns).attrib['d'] == route_path
assert {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (SOURCE_SVG,SOURCE_NOTES)} == SOURCE_HASHES
for pin in view['pins']:
    projected = xy(pin['longitude'], pin['latitude'])
    assert abs(projected[0]-pin['x']) < .011 and abs(projected[1]-pin['y']) < .011
print(json.dumps({'file':'explorer-base.svg','bytes':(HERE/'explorer-base.svg').stat().st_size,'rivers':[r['name'] for r in river_manifest],'waterLabels':water_labels,'validation':'Original land/route paths unchanged; original source files unchanged; pin transform unchanged.'}, indent=2))
