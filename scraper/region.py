from bs4 import BeautifulSoup
from contextlib import contextmanager
from dataclasses import dataclass
from io import BytesIO
from psycopg2.extras import execute_values
from requests import get
from typing import Optional
from zipfile import ZipFile
import re, pycountry, psycopg2, os, shapefile, json

# --- CONSTANTS --- #

country_geo_url = 'https://www.naturalearthdata.com/http//www.naturalearthdata.com/download/10m/cultural/ne_10m_admin_0_countries_lakes.zip'
province_geo_url = 'https://www.naturalearthdata.com/http//www.naturalearthdata.com/download/10m/cultural/ne_10m_admin_1_states_provinces_lakes.zip'
wikidata_url = 'https://query.wikidata.org/sparql'

exceptions = { 'CC': 'Cocos Islands'
             , 'CF': 'Central African Republic'
             , 'CI': 'Ivory Coast'
             , 'CD': 'DR Congo'
             , 'DO': 'Dominican Republic'
             , 'FK': 'Falkland Islands'
             , 'FM': 'Federated States of Micronesia'
             , 'IR': 'Iran'
             , 'KR': 'South Korea'
             , 'LA': 'Laos'
             , 'MF': 'Saint Martin'
             , 'KP': 'North Korea'
             , 'PS': 'Palestine'
             , 'SX': 'Sint Maarten'
             , 'SY': 'Syria'
             , 'VA': 'Vatican City'
             , 'VG': 'British Virgin Islands'
             , 'VI': 'U.S. Virgin Islands'
             }

excluded_regions = { 'CN-MO', 'CN-HK', 'CN-TW' }

# --- Fetch Region Names and IDs --- #

@dataclass
class Region:
    iso: str
    name: str
    geometry: Optional[str] = None

def is_bad_country_name(name):
    words = name.split()
    if ',' in name and ' and ' not in name: return True
    if words and words[-1] and words[-1][0].upper() != words[-1][0]: return True
    if words and words[0] and words[0][0].upper() != words[0][0]: return True
    if 'Republic' in words: return True
    if '(' in name or ')' in name: return True
    return False

def is_bad_subdivision_name(name):
    words = name.split()
    if ',' in name and ' and ' not in name: return True
    if '(' in name or ')' in name: return True
    return False

regions = {}

for country in pycountry.countries:
    iso  = country.alpha_2
    name = country.common_name if hasattr(country, 'common_name') else country.name

    if iso in exceptions:
        name = exceptions[iso]
    elif is_bad_country_name(name):
        print('[ugly name]', iso, '|', name)

    regions[iso] = Region(iso=iso, name=name)

for subdivision in pycountry.subdivisions:
    iso  = subdivision.code
    name = subdivision.name
    regions[iso] = Region(iso=iso, name=name)

for iso in excluded_regions:
    if iso in regions:
        regions.pop(iso)

# --- Fetch Geometry --- #

@contextmanager
def load_shapefile(url):
    with ZipFile(BytesIO(get(url).content)) as zf:
        names = zf.namelist()
        shp = next(x for x in names if x.endswith('.shp'))
        shx = next(x for x in names if x.endswith('.shx'))
        dbf = next(x for x in names if x.endswith('.dbf'))
        with zf.open(shp) as shp, zf.open(shx) as shx, zf.open(dbf) as dbf:
            with shapefile.Reader(shp=shp, shx=shx, dbf=dbf) as res:
                yield res

with load_shapefile(country_geo_url) as reader:
    iso_attr = [x[0] for x in reader.fields if x[0].upper() == 'ISO_A2']
    if iso_attr:
        iso_attr = iso_attr[0]
        for sr in reader.shapeRecords():
            iso = sr.record[iso_attr]
            if iso not in regions: continue
            geojson = sr.shape.__geo_interface__
            regions[iso].geometry = json.dumps(geojson)

with load_shapefile(province_geo_url) as reader:
    iso_attr = [x[0] for x in reader.fields if x[0].upper() == 'ISO_3166_2']
    if iso_attr:
        iso_attr = iso_attr[0]
        for sr in reader.shapeRecords():
            iso = sr.record[iso_attr]
            if iso not in regions: continue
            geojson = sr.shape.__geo_interface__
            regions[iso].geometry = json.dumps(geojson)

# --- Fetch Population Data --- #

q = '''
SELECT DISTINCT ?region ?population ?date {
    {
        ?country p:P1082 [ ps:P1082 ?population ; pq:P585 ?date ] .
        ?country wdt:P297 ?region .
    } UNION {
        ?province p:P1082 [ ps:P1082 ?population ; pq:P585 ?date ] .
        ?province wdt:P300 ?region .
    }

    FILTER (YEAR(?date) >= 2000)
}
'''

pops = {}
json = get(wikidata_url, params={ 'query': q, 'format': 'json' }).json()

for result in json['results']['bindings']:
    try:
        iso = result['region']['value']
        population = int(result['population']['value'])
        date = result['date']['value'][:10]
        if iso not in regions: continue
        pops[(iso, date)] = population
    except:
        print('[wikidata error]', result)

# --- Upload Data --- #

with psycopg2.connect(os.environ['DATABASE_URL']) as conn:
    with conn.cursor() as cur:
        execute_values(cur, '''
            INSERT INTO region(id, name, geometry)
            VALUES %s
            ON CONFLICT (id) DO UPDATE SET
                name = excluded.name,
                geometry = COALESCE(excluded.geometry, region.geometry)
        ''', [(r.iso, r.name, r.geometry) for r in regions.values()])

        execute_values(cur, '''
            INSERT INTO region_population(region, date, population)
            VALUES %s
            ON CONFLICT (region, date) DO UPDATE SET
                population = excluded.population
        ''', [(iso, date, pop) for (iso, date), pop in pops.items()])
