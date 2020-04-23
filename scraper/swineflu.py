import pandas as pd
import copy,psycopg2, os
from datetime import datetime
from pandas.io.html import read_html
from psycopg2.extras import execute_values
import country_converter as coco #I would use pycountry but it covers too little wikipedia names

url='https://en.wikipedia.org/wiki/2009_swine_flu_pandemic_tables'
tables = read_html(url,  attrs={"class":"wikitable sortable"})
cases = tables[0].rename({'Country or territory':'Country', 'April':'2009-04-01',
'May':'2009-05-01', 'June':'2009-06-01', 'July':'2009-07-01', 'August':'2009-08-01',
'Latest (9 August)':'2009-08-09_x'}, axis=1)
cases = cases.loc[3:176].set_index('Country').iloc[:,3:9]
deaths = tables[1].rename({'Apr': '2009-04-01', 'May':'2009-05-01', 'Jun':'2009-06-01', 
'Jul':'2009-07-01', 'Aug':'2009-08-01', 'Sep':'2009-09-09_y','Oct':'2009-10-09_y',
'Nov':'2009-11-09_y','Dec':'2009-12-09_y'}, axis=1)
deaths = deaths.loc[2:124].set_index('Country').iloc[:,3:12]
res = pd.merge(cases,deaths,on='Country', how='outer').fillna(0)

swine_cases = []
swine_deaths = []
for index, row in res.iterrows():
    col = 0
    iso = coco.convert(index,to='ISO2',include_obsolete=True)
    for value in row:
        entry = {}
        entry['location'] = iso
        entry['date'] = res.columns[col][:-2]
        if res.columns[col][-2:] == '_x':
            entry['cases'] = int(value)
            swine_cases.append(entry)
        else:
            e2 = copy.deepcopy(entry)
            e2['deaths'] = int(value)
            swine_deaths.append(e2)
        col = col + 1

df1 = pd.DataFrame(swine_cases)
df2 = pd.DataFrame(swine_deaths)
df = pd.merge(df1,df2, on=['location','date'],how='left', left_index=True,right_index=True)
swine_data = df.to_dict('records')

description = '''
Swine influenza is an infection caused by any one of several types of swine influenza viruses.
'''
#Not sure how popularity or reinfectability is determined
disease = {
    'id': 'H1N1',
    'name': 'Swine Flu',
    'long_name': 'Swine Influenza',
    'description': description,
    'reinfectable': False,
    'popularity': 2,
    'links': [
        [None, 'https://example.com', 'placeholder'],
    ]
    
}

conn =  psycopg2.connect(os.environ['DATABASE_URL'])
cur = conn.cursor()
cur.execute('''
    INSERT INTO disease(id, name, long_name, description, reinfectable, popularity)
    VALUES (%(id)s, %(name)s, %(long_name)s, %(description)s, %(reinfectable)s, %(popularity)s)
    ON CONFLICT (id) DO UPDATE SET
        name = excluded.name,
        long_name = excluded.long_name,
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
    (region, disease['id'], uri, desc)
    for region, uri, desc in disease['links']
])
execute_values(cur, '''
    INSERT INTO disease_stats(disease, region, date, cases, deaths)
        SELECT disease, region, date, cases::BIGINT, deaths::BIGINT
        FROM (VALUES %s)
            AS raw(disease, region, date, cases, deaths)
        WHERE EXISTS(SELECT 1 FROM region WHERE region.id = raw.region)
    ON CONFLICT (disease, region, date) DO UPDATE SET
        cases = COALESCE(excluded.cases, disease_stats.cases),
        deaths = COALESCE(excluded.deaths, disease_stats.deaths)
''', [
    (disease['id'], e['location'], datetime.strptime(e['date'], '%Y-%m-%d'), e['cases'], e['deaths'])
    for e in swine_data])
conn.commit()
