function get(f) {
    let cache = {};

    async function go(...args) {
        let url = f(...args);
        if (cache.hasOwnProperty(url)) return cache[url];

        let res = await fetch(url);
        if (!res.ok) throw new Error(`request failed with status ${res.status}`);

        let json = freeze(await res.json());

        if (!cache.hasOwnProperty(url)) cache[url] = json;
        return json;
    }

    return go;
}

function freeze(obj) {
    if (typeof obj != 'object') return obj;

    for (let key in obj) {
        if (!obj.hasOwnProperty(key)) continue;
        freeze(obj[key]);
    }

    return Object.freeze(obj);
}

export default {
    region: {
        list: get(() => '/data/region'),
        subregions: get(id => id ? `/data/region/subregions/${id}` : '/data/region/subregions'),
        current: get(() => '/data/region/current'),
    },

    disease: {
        list: get(() => '/data/disease'),
        get: get(id => `/data/disease/${id}`),
        inRegion: get((id, region) => `/data/disease/${id}/${region}`),
    },
};
