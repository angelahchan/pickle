import Header from './header';
import { useLoader, Loading, Error } from '../loading';
import { useParams } from 'react-router';
import data from '../../data';
import React from 'react';
import { MapControl, GeoJSON, Map as LMap, TileLayer } from 'react-leaflet';
import centroid from '@turf/centroid';
import Control from 'react-leaflet-control';

const MAP_STYLE = {
    width: '100%',
    maxWidth: '1440px',
    height: '75vh',
    maxHeight: '900px',
    border: '1px solid black',
    backgroundColor: 'white',
    margin: '1em auto',
};

const REGION_STYLE = {
    color: '#000',
    fillColor: '#F00',
    weight: 0,
    fillOpacity: 0,
};

const INFOBOX_STYLE = {
    borderRadius: '4px',
    backgroundColor: 'white',
    borderRadius: '4px',
    fontSize: 'initial',
    padding: '8px 1em',
    border: '1px solid black',
};

// At least this many people per million need to be infected for full red.
const MIN_WORST = 1000;

export default function Map({}) {
    let [country, setCountry] = React.useState(null);
    let [selectedRegion, setSelectedRegion] = React.useState(null);
    let map = React.useRef(null);

    let { disease: id } = useParams();

    let [err1, disease] = useLoader(() => data.disease.get(id), [ id ]);
    let [err2, rawc] = useLoader(() => data.region.subregions(null), []);
    let [err3, raws] = useLoader(() => (
        country
            ? data.region.subregions(country.properties.id)
            : Promise.resolve(null)
    ), [ country ]);

    let countries = React.useMemo(() => (
        disease && rawc
            ? getGeometries(disease, rawc)
            : []
    ), [disease, rawc]);

    let subdivisions = React.useMemo(() => (
        disease && raws
            ? getGeometries(disease, raws)
            : []
    ), [disease, raws]);

    if (err1 != null) return <Error error={err1} />;
    if (err2 != null) return <Error error={err2} />;
    if (err3 != null) return <Error error={err3} />;
    if (disease == null) return <Loading />;

    let current = country ? subdivisions : countries;
    let loading = (country ? raws : rawc) == null;

    let worst = getMaxActivePerMillion(current);
    if (worst != null) worst = Math.max(worst, MIN_WORST);

    let style = feature => {
        let properties = feature.properties;
        if (properties.activePerMillion == null) return REGION_STYLE;
        let k = properties.activePerMillion / worst;
        return { ...REGION_STYLE, fillOpacity: 0.75*Math.sqrt(k) };
    };

    let onEachFeature = (feature, layer) => {
        layer.setStyle(style(feature));

        layer.on('mouseover', e => {
            layer.setStyle({ weight: 1 });
            layer.bringToFront();
            setSelectedRegion(feature);
        });

        layer.on('mouseout', e => {
            layer.setStyle({ weight: 0 });
            setSelectedRegion(cur => {
                if (cur == null) return null;
                if (cur.properties.id == feature.properties.id) return null;
                return cur;
            });
        });

        if (feature.properties.subdivisible) {
            layer.on('click', e => {
                let zoom = Math.max(4, map.current.leafletElement.getZoom());
                map.current.leafletElement.flyTo(feature.properties.center, zoom);
                setCountry(feature);
                setSelectedRegion(null);
            });
        }
    };

    let onZoomEnd = e => {
        if (country && map.current.leafletElement.getZoom() < 4) {
            setCountry(null);
        }
    };

    let infobox;
    if (selectedRegion != null) {
        infobox = <RegionInformation region={selectedRegion} />;
    } else if (loading) {
        infobox = <LoadingInformation />;
    } else {
        infobox = <WaitingInformation country={country} />;
    }

    let geojson = null;
    if (!loading) {
        let key = country ? country.properties.id : '';
        geojson = (
            <GeoJSON
                key={key}
                data={current}
                onEachFeature={onEachFeature}
            />
        );
    }

    return (
        <div>
            <Header title={`The ${disease.name} Map`} />
            <LMap
                ref={map}
                center={[0, 0]}
                zoom={2}
                style={MAP_STYLE}
                attributionControl={false}
                onZoomEnd={onZoomEnd}
            >
                <TileLayer url='https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png'/>
                <Control position='topright'>
                    <div style={INFOBOX_STYLE}>{infobox}</div>
                </Control>
                {geojson}
            </LMap>
        </div>
    );
}

function getMaxActivePerMillion(geometries) {
    return geometries.reduce((acc, feature) => {
        let properties = feature.properties;
        if (properties == null) return acc;
        if (properties.activePerMillion == null) return acc;
        if (acc == null) return properties.activePerMillion;
        return Math.max(properties.activePerMillion, acc);
    }, null);
}

function getGeometries(disease, subregions) {
    let geometries = [];
    let data = {}

    for (let region of subregions) {
        data[region.id] = {
            id: region.id,
            name: region.name,
            subdivisible: false,
        };

        if (region.geometry != null) {
            let geometry = JSON.parse(region.geometry);

            let [lon, lat] = centroid(geometry).geometry.coordinates;
            data[region.id].center = { lat, lon };

            geometries.push({
                type: 'Feature',
                properties: data[region.id],
                geometry,
            });
        }
    }

    for (let { region: id, cases, deaths, recoveries, population } of disease.stats) {
        if (id.includes('-')) {
            let first = id.split('-')[0];
            if (data.hasOwnProperty(first)) {
                data[first].subdivisible = true;
            }
        }

        if (!data.hasOwnProperty(id)) continue;

        let active = (cases == null)
            ? null
            : Math.max(0, cases - (recoveries || 0) - (deaths || 0));

        let activePerMillion = (active == null || population == null)
            ? null
            : 1000000 * active / population;

        data[id].cases = cases;
        data[id].deaths = deaths;
        data[id].recoveries = recoveries;
        data[id].population = population;
        data[id].active = active;
        data[id].activePerMillion = activePerMillion;
    }

    return geometries;
}

function LoadingInformation({}) {
    return <p>Loading…</p>;
}

function WaitingInformation({ country }) {
    if (country != null) {
        return (
            <div>
                <h2>{country.properties.name}</h2>
                <p>Hover over a region for more information.</p>
                <p>Zoom out to return to the rest of the world.</p>
            </div>
        );
    } else {
        return (
            <div>
                <p>Hover over a country for more information.</p>
            </div>
        );
    }
}

function RegionInformation({ region }) {
    let props = region.properties;

    let texts = [
        ['actives', props.activePerMillion, 'active case per million people', 'active cases per million people', '#D00'],
        ['cases', props.cases, 'case', 'cases', undefined],
        ['deaths', props.deaths, 'death', 'deaths', undefined],
        ['recoveries', props.recoveries, 'recovery', 'recoveries', undefined],
    ];

    let divs = texts.map(([ key, value, sing, plur, color ]) => {
        if (value == null) return null;
        return <div style={{ color }}>{Math.round(value)} {value == 1 ? sing : plur}</div>;
    });

    if (divs.every(x => x == null)) divs.push(<div>No data.</div>);

    return (
        <div>
            <h2>{ props.name || 'Unknown Region' }</h2>
            <p>{divs}</p>
            {
                props.subdivisible
                    ? <p>Click for a state-by-state breakdown.</p>
                    : null
            }
        </div>
    );
}
