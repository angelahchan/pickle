import React from 'react';
import { Link } from 'react-router-dom';
import { useLoader, Error, Loading } from './loading';
import data from '../data';

export default function DiseaseSelect({}) {
    let [err, res] = useLoader(data.disease.list, []);

    if (err) return <Error error={err} />;
    if (res == null) return <Loading />;

    let select = (res != null)
        ? Array
            .from(res)
            .sort((x, y) => y.popularity - x.popularity)
            .map(x => (
                <li key={x.id}>
                    <Link to={`/disease/${x.id}/`}>{x.name}</Link> — {x.long_name}
                </li>
            ))
        : <li>Loading…</li>;

    return (
        <div>
            <header>
                <h1>Pickle</h1>
            </header>
            <article>
                <p>
                    Pickle is a website that keeps you up-to-date on the latest pandemic — we give
                    you the latest advice, news and stats in your local region.
                </p>
                <p>
                    Which disease do you want to know more about?
                </p>
                <ul>{select}</ul>
            </article>
        </div>
    );
}
