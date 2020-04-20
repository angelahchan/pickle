import React from 'react';
import {
    BrowserRouter as Router,
    Switch,
    Route,
    Link,
    useParams,
    Redirect,
} from 'react-router-dom';

import data from '../data'

import DiseaseSelect from './disease-select';
import DiseaseMap from './disease/map';
import DiseaseFaq from './disease/faq';
import DiseaseCompare from './disease/compare';
import DiseaseNews from './disease/news';
import NotFound from './not-found';
import { useLoader, Loading, Error } from './loading';

export default function Main({}) {
    return (
        <Router>
            <Switch>
                <Route exact path='/'>
                    <RedirectToDisease />
                </Route>

                <Route exact path='/disease/:disease'>
                    <RedirectToDisease />
                </Route>

                <Route exact path='/disease/:disease/news'>
                    <RedirectToDisease path="/news" />
                </Route>



                <Route exact path='/disease-select'>
                    <DiseaseSelect />
                </Route>



                <Route exact path='/disease/:disease/map'>
                    <DiseaseMap />
                </Route>

                <Route exact path='/disease/:disease/in/:region'>
                    <DiseaseFaq />
                </Route>

                <Route exact path='/disease/:disease/in/:region/vs/:other'>
                    <DiseaseCompare />
                </Route>

                <Route exact path='/disease/:disease/in/:region/news'>
                    <DiseaseNews />
                </Route>



                <Route>
                    <NotFound />
                </Route>
            </Switch>
        </Router>
    );
}

function RedirectToDisease({ path }) {
    let { disease } = useParams();
    let [err1, diseases] = useLoader(data.disease.list, []);
    let [err2, region] = useLoader(data.region.current, []);

    if (err1 != null) return <Error error={err1} />;
    if (err2 != null) return <Error error={err2} />;

    if (disease == null) {
        if (diseases == null) return <Loading />;

        disease = diseases.reduce((acc, cur) => {
            if (acc == null) return cur;
            if (acc.popularity < cur.popularity) return cur;
            return acc;
        }, null);

        if (disease == null) {
            return <Error error='There are no diseases in our system right now.' />;
        }

        disease = disease.id;
    }

    if (region == null) return <Loading />;
    return (
        <Redirect to={`/disease/${disease}/in/${region}${path || ''}`} />
    );
}
