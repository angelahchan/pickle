import React from 'react';

export function Loading({}) {
    return (
        <header>
            <h1>Loading…</h1>
        </header>
    );
}

export function Error({ error }) {
    return (
        <header>
            <h1>Sorry, an error occurred.</h1>
        </header>
    );
}

export function useLoader(f, deps) {
    deps.length; // Make sure that it's defined.

    let [err, setErr] = React.useState(null);
    let [res, setRes] = React.useState(null);

    React.useEffect(() => {
        setErr(null);
        setRes(null);
        f().then(x => setRes(x)).catch(e => setErr(e));
    }, deps);

    return [err, res];
};
