# Pickle

Pickle informs you about pandemics.

## Getting Started

1. Install the requirements.
2. Run `npm install` in `/client`.
3. Run `pip install -r requirements.txt` in `/scraper`.
4. Set up a new PostgreSQL database and initialize it with `schema.sql`.
5. Run the scrapers to populate the database.
6. Set the environment variables.
7. Run `make run` in the project root to start the server.

## Requirements

| Requirement     | Tested Version |
|-----------------|----------------|
| Rust            | 1.42.0         |
| PostgreSQL      | 12.2           |
| GNU Make        | 4.3            |
| Python          | 3.8.2          |
| Parcel          | 1.12.4         |

## Environment Variables

| Variable       | Example                            | Required? | Description                                                                            |
|----------------|------------------------------------|-----------|----------------------------------------------------------------------------------------|
| `MAXMIND_KEY`  | `369DBF124085C7A`                  | Yes       | The key used to download the MaxMind GeoLite2 database, which does the IP geolocation. |
| `DATABASE_URL` | `postgres://localhost:5432/pickle` | Yes       | The URL to the main database.                                                          |
| `PORT`         | `8000`                             | No        | The port for the server to listen on.                                                  |

## Endpoints

```
FRONTEND
    /
        redirect to /{most popular disease}/in/{guessed user region}

    /:disease
        redirect to /:disease/in/{guessed user region}

    /:disease/map
        latest disease statistics by country

    /:disease/in/:region
        summary of the disease in that region

        If there is no data for this region, it offers to redirect the user
        to the encompassing region.

    /:disease/in/:region/vs/:other-region
        comparison between how these two countries are handling the disease

    /about
        website description and contact information

DATA
    /data/region
        region list

        [ { id   : string -- "US"
          , name : string -- "United States"
          }
        ]

    /data/region/current
        guess the current region by the sender's IP address

        string

    /data/region/subregions/:id
        subregion list; leave :id empty for a list of countries

        [ { id       : string  -- "US"
          , name     : string  -- "United States"
          , geometry : object? -- { ... GeoJSON geometry ... }
          }
        ]

    /data/disease
        list of diseases

        [ { id         : string
          , name       : string
          , popularity : number
          }
        ]

    /data/disease/:disease
        disease description with stats for each affected region

        { id           : string
        , name         : string
        , description  : string
        , reinfectable : boolean
        , popularity   : number
        , stats        : [ { region     : string
                           , cases      : number?
                           , deaths     : number?
                           , recoveries : number?
                           , population : number?
                           }
                         ]
        }

    /data/disease/:disease/in/:region
        disease stats over time within the given region

        { id         : string
        , links      : [ { uri         : string
                         , description : string
                         }
                       ]
        , stats      : [ { date       : string
                         , cases      : number?
                         , deaths     : number?
                         , recoveries : number?
                         }
                       ]
        , population : [ { date       : string
                         , population : number?
                         }
                       ]
        }

    /data/disease/:disease/in/:region/news
        latest news about the disease in that region

        [ { title     : string
          , url       : string
          , source    : string
          , published : string
          }
        ]
```
