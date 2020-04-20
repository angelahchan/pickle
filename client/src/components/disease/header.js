import React from 'react';
import { useParams, NavLink } from 'react-router-dom';

export default function Header({ title }) {
    let { disease, region } = useParams();

    region = region || '';

    let links = [
        [ 'FAQ'            , `/disease/${disease}/in/${region}`      , `/disease/${disease}`      ],
        [ 'Map'            , `/disease/${disease}/map`               , `/disease/${disease}/map`  ],
        [ 'News'           , `/disease/${disease}/in/${region}/news` , `/disease/${disease}/news` ],
        [ 'Other Diseases' , `/disease-select`                       , `/disease-select`          ],
    ];

    let nav = [];
    for (let i = 0; i < links.length; i++) {
        if (i > 0) nav.push(<span key={2*i}> · </span>);
        let [text, url1, url2] = links[i];
        nav.push(<NavLink exact to={region ? url1 : url2} key={2*i + 1}>{text}</NavLink>);
    }

    return (
        <header>
            <h1>{title}</h1>
            <nav>{nav}</nav>
        </header>
    );
}
