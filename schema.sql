-- # Notes
--
-- Some regions have codes in both ISO 3166-1 and ISO 3166-2. For these
-- regions, use the ISO 3166-1 code.

CREATE TABLE region
    ( id        TEXT PRIMARY KEY -- ISO 3166-1 alpha-2 / ISO 3166-2 / empty string
    , name      TEXT NOT NULL    -- a short English name of the region, in title-case
    , geometry  TEXT             -- a low-resolution GeoJSON geometry for the region
    );

CREATE TABLE disease
    ( id            TEXT PRIMARY KEY -- an uppercase codename for the disease
    , name          TEXT NOT NULL    -- a human-readable name for the disease
    , description   TEXT NOT NULL    -- a CommonMark article describing the disease
    , reinfectable  BOOLEAN NOT NULL -- whether a significant number of people will be infected multiple times
    , popularity    REAL NOT NULL    -- the "ranking" of this disease
    );

CREATE TABLE disease_stats
    ( date        DATE NOT NULL
    , region      TEXT NOT NULL REFERENCES region(id)
    , disease     TEXT NOT NULL REFERENCES disease(id)
    , cases       BIGINT
    , deaths      BIGINT
    , recoveries  BIGINT
    , PRIMARY KEY (disease, region, date)
    );

CREATE TABLE disease_link
    ( region      TEXT NOT NULL REFERENCES region(id)
    , disease     TEXT NOT NULL REFERENCES disease(id)
    , uri         TEXT NOT NULL -- a URI that points to a human-readable resource
    , description TEXT NOT NULL -- the resource's purpose (CommonMark; first letter uncapitalized; no full stop)
    );

CREATE TABLE region_population
    ( date        DATE NOT NULL
    , region      TEXT NOT NULL REFERENCES region(id)
    , population  BIGINT
    , PRIMARY KEY (region, date)
    );
