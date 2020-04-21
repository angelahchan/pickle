import Header from './header';
import { useLoader, Loading, Error } from '../loading';
import { useParams } from 'react-router';
import data from '../../data';
import React from 'react';

// TODO: The news.
export default function News({}) {
    let { disease: id } = useParams();
    let [err, disease] = useLoader(() => data.disease.get(id), [ id ]);

    // https://news.google.com/rss/search
    // q=' '.join(country.name, disease.name) (+-encoded)
    // gl=country.id
    //
    // this should get you an RSS feed of news items, and you can use that

    if (err != null) return <Error error={err} />;
    if (disease == null) return <Loading />;

    return (
        <Header title={`${disease.name} News`} />
    );
}
