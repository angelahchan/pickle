-- # Notes
--
-- * Some regions have codes in both ISO 3166-1 and ISO 3166-2. For these
--   regions, use the ISO 3166-1 code.
-- * The `disease` and `disease_link` table is updated entirely by hand. Other
--   tables are updated using helper scripts.

CREATE TABLE region
    ( id        TEXT PRIMARY KEY -- ISO 3166-1 alpha-2 / ISO 3166-2 / empty string
    , name      TEXT NOT NULL    -- A short English name of the region, in title-case.
    , the       TEXT NOT NULL    -- Whether "the" should appear before the name in a sentence.
    , geometry  TEXT NOT NULL    -- A low-resolution GeoJSON geometry for the region.
    );

CREATE TABLE disease
    ( id            TEXT PRIMARY KEY -- A codename for the disease.
    , name          TEXT NOT NULL    -- A human-readable name for the disease.
    , description   TEXT NOT NULL    -- A one-sentence summary of the disease.
    , reinfectable  BOOLEAN NOT NULL -- Whether a significant number of people will be infected multiple times.
    , popularity    FLOAT NOT NULL   -- The "ranking" of this disease.
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
    , uri         TEXT NOT NULL
    , description TEXT NOT NULL
    );

CREATE TABLE region_population
    ( date        DATE NOT NULL
    , region      TEXT NOT NULL REFERENCES region(id)
    , population  BIGINT
    , PRIMARY KEY (region, date)
    );
