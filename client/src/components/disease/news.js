import Header from './header';
import { useLoader, Loading, Error } from '../loading';
import { useParams } from 'react-router';
import data from '../../data';
import React from 'react';
import formatRelative from 'date-fns/formatRelative';
import RegionSelect from '../region-select';
import { useHistory } from 'react-router-dom';

export default function News({}) {
    let { disease: id, region } = useParams();
    let history = useHistory();

    let [err1, news] = useLoader(() => data.disease.newsInRegion(id, region), [ id, region ]);
    let [err2, disease] = useLoader(() => data.disease.get(id), [ id ]);

    if (err1 != null) return <Error error={err1} />;
    if (err2 != null) return <Error error={err2} />;
    if (disease == null || news == null) return <Loading />;

    let setRegion = newRegion => {
        history.push(`/disease/${id}/in/${newRegion}/news`);
    };

    return (
        <div>
            <Header title={`${disease.name} News`} />
            <div className='reading-width'>
                <div>
                    <RegionSelect showing={'news'} region={region} setRegion={setRegion} />
                </div>
                <div>{ news.map(x => <Article article={x} />) }</div>
            </div>
        </div>
    );
}

function Article({ article }) {
    let now = Date.now();
    let reltime = formatRelative(Date.parse(article.published), Date.now());

    return (
        <summary>
            <hr />
            <h3><a href={article.url} target='_blank'>{article.title}</a></h3>
            <small><cite>{article.source}</cite> · <time datetime={article.published}>{reltime}</time></small>
            <p>{article.description}</p>
        </summary>
    );
}
