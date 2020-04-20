import Header from './header';
import { useLoader, Loading, Error } from '../loading';
import { useParams } from 'react-router';
import data from '../../data';
import React from 'react';

// TODO: The news.
export default function News({}) {
    let { disease: id } = useParams();
    let [err, disease] = useLoader(() => data.disease.get(id), [ id ]);

    if (err != null) return <Error error={err} />;
    if (disease == null) return <Loading />;

    return (
        <Header title={`${disease.name} News`} />
    );
}
