from datetime import date
from collections import defaultdict
from psycopg2.extras import execute_values
from requests import get
import psycopg2, os, re

result = defaultdict(lambda: defaultdict(lambda: { k: None for k in ('cases', 'deaths', 'recoveries') }))

# --- TODO: Description and Links --- #

description = '''
COVID-19 is a disease and it sucks.
'''

disease = {
    'id': 'COVID-19',
    'name': 'COVID-19',
    'description': description,
    'reinfectable': False,
    'popularity': 1,
    'links': [
        ['', 'https://example.com', 'something helpful'],
        ['AU', 'https://example.com', 'something helpful, too'],
        ['AU-NSW', 'https://example.com', 'more helpful stuff']
    ]
}

# --- Corona Data Scraper --- #

data = get('https://coronadatascraper.com/timeseries-byLocation.json').json()

for loc in data.values():
    if 'featureId' not in loc or 'dates' not in loc:
        continue

    feature_id = loc['featureId']
    dates = loc['dates']

    if feature_id.startswith('iso1:') or feature_id.startswith('iso2:'):
        iso = feature_id[5:]
    else:
        continue

    for d, v in dates.items():
        try:
            d = date.fromisoformat(d)
        except ValueError:
            continue

        try:
            data = {
                k1: int(v[k2])
                for k1, k2 in (
                    ('cases', 'cases'),
                    ('deaths', 'deaths'),
                    ('recoveries', 'recovered')
                )
                if v.get(k2) not in (None, '')
            }
        except ValueError:
            continue

        if not data: continue

        result[iso][d].update(data)

# --- Upload --- #

with psycopg2.connect(os.environ['DATABASE_URL']) as conn:
    with conn.cursor() as cur:
        cur.execute('''
            INSERT INTO disease(id, name, description, reinfectable, popularity)
                VALUES (%(id)s, %(name)s, %(description)s, %(reinfectable)s, %(popularity)s)
                ON CONFLICT (id) DO UPDATE SET
                    name = excluded.name,
                    description = excluded.description,
                    reinfectable = excluded.reinfectable,
                    popularity = excluded.popularity
        ''', disease)

        cur.execute('''
            DELETE FROM disease_link WHERE disease = %(name)s
        ''', disease)

        execute_values(cur, '''
            INSERT INTO disease_link(region, disease, uri, description)
                VALUES %s
        ''', [
            (region, disease['name'], uri, desc)
            for region, uri, desc in disease['links']
        ])

        execute_values(cur, '''
            INSERT INTO disease_stats(disease, region, date, cases, deaths, recoveries)
                SELECT disease, region, date, cases::BIGINT, deaths::BIGINT, recoveries::BIGINT
                    FROM (VALUES %s)
                        AS raw(disease, region, date, cases, deaths, recoveries)
                    WHERE EXISTS(SELECT 1 FROM region WHERE region.id = raw.region)
                ON CONFLICT (disease, region, date) DO UPDATE SET
                    cases = COALESCE(excluded.cases, disease_stats.cases),
                    deaths = COALESCE(excluded.deaths, disease_stats.deaths),
                    recoveries = COALESCE(excluded.recoveries, disease_stats.recoveries)
        ''', [
            (disease['id'], iso, date, v['cases'], v['deaths'], v['recoveries'])
            for iso in result
            for date, v in result[iso].items()
        ])
