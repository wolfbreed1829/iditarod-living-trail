"""Rebuild orientation assets from public primary geographic data.

Run with Python 3 and requests. No credentials or paid services are required.
The spherical Alaska Albers projection is for an illustrated overview, not survey work.
"""
from pathlib import Path
import concurrent.futures
import json
import math
import requests

HERE = Path(__file__).resolve().parent
NE = 'https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_50m_admin_1_states_provinces.geojson'
ROUTE = 'https://services1.arcgis.com/fBc8EJBxQRMcHlei/arcgis/rest/services/Iditarod_National_Historic_Trail_BLM_Official_Temp/FeatureServer/0'
GNIS = 'https://carto.nationalmap.gov/arcgis/rest/services/geonames/MapServer'
ITEM = 'https://www.arcgis.com/sharing/rest/content/items/37787244e85c450c8247c34eb221954c'

def fetch(url, params=None):
    response = requests.get(url, params=params, timeout=60)
    response.raise_for_status()
    return response.json()

queries = {
    'natural_earth': (NE, None),
    'route': (ROUTE + '/query', {'where':'1=1','outFields':'*','outSR':4326,'f':'geojson'}),
    'towns': (GNIS + '/3/query', {'where':"state_alpha='AK' AND gaz_name IN ('Seward','Girdwood','Knik','McGrath','Iditarod','Unalakleet','Nome')", 'outFields':'gaz_id,gaz_name,gaz_featureclass,state_alpha','outSR':4326,'returnGeometry':'true','f':'json'}),
    'passes': (GNIS + '/5/query', {'where':"state_alpha='AK' AND gaz_name IN ('Rainy Pass','Crow Pass')", 'outFields':'gaz_id,gaz_name,gaz_featureclass,state_alpha','outSR':4326,'returnGeometry':'true','f':'json'}),
    'route_metadata': (ITEM, {'f':'json'}),
}
with concurrent.futures.ThreadPoolExecutor(max_workers=5) as pool:
    futures = {key:pool.submit(fetch,*args) for key,args in queries.items()}
    data = {key:future.result() for key,future in futures.items()}

alaska = next(f for f in data['natural_earth']['features'] if f['properties']['name']=='Alaska')
alaska['properties'] = {'name':'Alaska','source':'Natural Earth 1:50m admin-1 states/provinces','license':'Public domain'}
alaska_collection = {'type':'FeatureCollection','features':[alaska]}
route = data['route']
route.pop('crs',None)

def compact_write(name, content):
    (HERE/name).write_text(json.dumps(content,separators=(',',':'),ensure_ascii=False)+'\n',encoding='utf-8')

compact_write('alaska-natural-earth.geojson', alaska_collection)
compact_write('iditarod-historic-route.geojson', route)

# Equal-area Albers projection on a sphere: standard parallels 55/65 N,
# origin 50 N, central meridian 154 W. SVG uses a uniform scale and inverted y.
rad=math.radians
n=(math.sin(rad(55))+math.sin(rad(65)))/2
c=math.cos(rad(55))**2+2*n*math.sin(rad(55))
rho0=math.sqrt(c-2*n*math.sin(rad(50)))/n
def project(lon,lat):
    delta=(lon+154+180)%360-180
    rho=math.sqrt(c-2*n*math.sin(rad(lat)))/n
    theta=n*rad(delta)
    return [6371008.8*rho*math.sin(theta),6371008.8*(rho0-rho*math.cos(theta))]

def distance_squared(p,a,b):
    dx=b[0]-a[0];dy=b[1]-a[1]
    t=max(0,min(1,((p[0]-a[0])*dx+(p[1]-a[1])*dy)/(dx*dx+dy*dy))) if dx or dy else 0
    return (p[0]-a[0]-t*dx)**2+(p[1]-a[1]-t*dy)**2

def simplify(points,tolerance):
    if len(points)<3:return points
    index,d=max(((i,distance_squared(points[i],points[0],points[-1])) for i in range(1,len(points)-1)),key=lambda v:v[1])
    if d>tolerance*tolerance:return simplify(points[:index+1],tolerance)[:-1]+simplify(points[index:],tolerance)
    return [points[0],points[-1]]

def path(points,closed=False):
    return 'M'+'L'.join(f'{x:.2f},{y:.2f}' for x,y in points)+('Z' if closed else '')

pins=[]
for feature in data['towns']['features']+data['passes']['features']:
    attr=feature['attributes'];lon,lat=feature['geometry']['points'][0]
    if lon>0:continue # Exclude the unrelated Aleutian Rainy Pass.
    pins.append({'id':attr['gaz_name'].lower().replace(' ','-'),'name':attr['gaz_name'],'longitude':round(lon,6),'latitude':round(lat,6),'gnisId':attr['gaz_id'],'featureClass':attr['gaz_featureclass'],'source':f"{GNIS}/{5 if attr['gaz_featureclass']=='Gap' else 3}/query?where=gaz_id%3D{attr['gaz_id']}&outFields=*&outSR=4326&f=pjson",'coordinateNote':'First official GNIS location point; approximate label placement, not trail access or a trailhead.'})

views={}
for name,bounds,width,height in [('trail-orientation',[-168.3,58.7,-144.5,67.6],900,700),('alaska-overview',[-170,57,-139.5,72],900,820)]:
    west,south,east,north=bounds
    edge=[]
    for i in range(101):
        t=i/100
        edge.extend([project(west+(east-west)*t,south),project(west+(east-west)*t,north),project(west,south+(north-south)*t),project(east,south+(north-south)*t)])
    xmin=min(p[0] for p in edge);xmax=max(p[0] for p in edge);ymin=min(p[1] for p in edge);ymax=max(p[1] for p in edge)
    scale=min((width-20)/(xmax-xmin),(height-20)/(ymax-ymin))
    xoffset=(width-scale*(xmax-xmin))/2-scale*xmin
    yoffset=(height-scale*(ymax-ymin))/2+scale*ymax
    def xy(coord):
        x,y=project(*coord)
        return [x*scale+xoffset,-y*scale+yoffset]
    polygon_coordinates=alaska['geometry']['coordinates'] if alaska['geometry']['type']=='MultiPolygon' else [alaska['geometry']['coordinates']]
    land_paths=[]
    for polygon in polygon_coordinates:
        ring_paths=[]
        for ring in polygon:
            transformed=[xy(p) for p in ring]
            # Discard small/offscreen islands, retaining mainland and visible coast.
            if max(p[0] for p in transformed)<-10 or min(p[0] for p in transformed)>width+10 or max(p[1] for p in transformed)<-10 or min(p[1] for p in transformed)>height+10:continue
            ring_paths.append(path(simplify(transformed,.65),True))
        if ring_paths:land_paths.append(''.join(ring_paths))
    route_paths=[]
    for feature in route['features']:
        geom=feature['geometry']
        lines=[geom['coordinates']] if geom['type']=='LineString' else geom['coordinates']
        for line in lines:route_paths.append(path(simplify([xy(p) for p in line],.35)))
    land=''.join(f'<path d="{p}"/>' for p in land_paths)
    route_d=''.join(route_paths)
    svg=f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {width} {height}" fill="none" role="img" aria-labelledby="map-title map-description">
  <title id="map-title">Iditarod National Historic Trail geographic overview</title>
  <desc id="map-description">Alaska coastline with the BLM historic trail alignment from Seward to Nome. Regional orientation only; this does not show connecting routes or a current race course and is not a navigation map.</desc>
  <defs><clipPath id="map-clip"><rect width="{width}" height="{height}"/></clipPath></defs>
  <g clip-path="url(#map-clip)">
    <g class="map-land" fill="#294c44" stroke="#567268" stroke-width="1.05" fill-rule="evenodd" stroke-linejoin="round">{land}</g>
    <path class="map-route-casing" d="{route_d}" stroke="#18372f" stroke-width="6" stroke-linecap="round" stroke-linejoin="round"/>
    <path class="map-route" d="{route_d}" stroke="#d29c6b" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round"/>
  </g>
</svg>'''
    (HERE/(name+'.svg')).write_text(svg,encoding='utf-8')
    views[name]={'file':name+'.svg','viewBox':f'0 0 {width} {height}','width':width,'height':height,'geographicFocusBounds':bounds,'boundsOrder':'west,south,east,north','projection':'Spherical Alaska Albers equal-area; standard parallels 55/65; central meridian -154; latitude origin 50; R=6371008.8m','transform':{'scale':scale,'xOffset':xoffset,'yOffset':yoffset,'formula':'svgX=projectedX*scale+xOffset; svgY=-projectedY*scale+yOffset'},'pins':[{**p,'x':round(xy([p['longitude'],p['latitude']])[0],2),'y':round(xy([p['longitude'],p['latitude']])[1],2)} for p in pins]}

manifest={
    'retrieved':'2026-11-06',
    'purpose':'An orientation map of the historic trail, not directions, an access map, or a current race route.',
    'recommendedVisibleCaption':'Historic trail overview · BLM route, Natural Earth coastline. For orientation only; not a navigation map.',
    'routeScope':'This BLM centerline represents the historic route. Its source explicitly excludes connecting routes and the dog sled race route. The historic network is broader than this single displayed alignment.',
    'editorialNotes':[
        {'text':'BLM describes a 1,000-mile main trail between Seward and Nome plus 1,400 miles of side and connecting trails. Do not label the single displayed alignment as the whole 2,400-mile network.','source':'https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod'},
        {'text':'The network crosses lands under multiple public and private owners. A line on this orientation map is not a statement of permission, safe conditions, or continuous year-round access.','source':'https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod'},
        {'text':'IHTA describes the trail as primarily a winter trail and highlights summer trails in Chugach State Park, Chugach National Forest, Girdwood and Eagle River. Seasonal controls should change the visitor recommendations rather than imply this whole historic alignment is a summer hiking route.','source':'https://www.iditarod100.org/plan-your-trip.html'},
        {'text':'The BLM brochure distinguishes the main historic route, connecting historic routes and race routes as three separate map layers. The SVG here contains only the first.','source':'https://www.blm.gov/sites/blm.gov/files/documents/files/PublicRoom_Alaska_2018-Iditarod-National-Historic-Trail-Brochure.pdf'},
        {'text':'Girdwood and Crow Pass are geographically close at this map scale. Use one Southcentral cluster pin, reveal details in a panel, or offset only the label with a leader line; do not move the geographic marker.'}
    ],
    'views':views,
    'sources':[
        {'title':'Natural Earth 1:50m states/provinces','url':NE,'license':'Public domain','terms':'https://www.naturalearthdata.com/about/terms-of-use/','changes':'Alaska selected; projected, simplified and clipped in SVG. GeoJSON retains the source geometry.'},
        {'title':'BLM Iditarod National Historic Trail centerline, hosted by NPS','url':ROUTE,'item':'https://www.arcgis.com/home/item.html?id=37787244e85c450c8247c34eb221954c','metadata':ITEM+'/info/metadata/metadata.xml','credit':'Kevin Keeler - Bureau of Land Management','licenseInfo':data['route_metadata'].get('licenseInfo'),'supplementalInformation':'Route segments and historic features recommended for inclusion under the 1986 INHT Comprehensive Management Plan and subsequent BLM Anchorage Field Office delineation. Historic centerline excludes connecting routes and dog sled race route. NPS hosts the data for web mapping. Consult original service/item metadata for full context.','changes':'Source centerline retained in GeoJSON; projected and simplified by 0.35 SVG pixels for display.'},
        {'title':'USGS The National Map Geographic Names Information System','url':GNIS,'license':'US public-domain geographic names data','changes':'Selected Alaska town and pass points; first official location when GNIS supplies multiple points.'},
        {'title':'USFWS index independently identifying the BLM service as the historic trail official route','url':'https://www.fws.gov/r7/nwr/Realty/data/LandMappers/Public/Help/HTML/OnlineGISDataServices.html'},
        {'title':'BLM Iditarod National Historic Trail','url':'https://www.blm.gov/programs/national-conservation-lands/national-scenic-and-historic-trails/iditarod'},
        {'title':'IHTA plan your trip','url':'https://www.iditarod100.org/plan-your-trip.html'}
    ]
}
(HERE/'map-sources.json').write_text(json.dumps(manifest,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
print(json.dumps({'files':[p.name for p in HERE.iterdir()],'focusPins':views['trail-orientation']['pins'],'routeFeatures':len(route['features'])},indent=2))
