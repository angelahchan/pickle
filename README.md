# Pickle

Pickle gives you simple summaries of pandemics.

## Endpoints

```
FRONTEND
    /
        redirect to /{most popular disease}/in/{guessed user region}

    /:disease/
        summary of the disease

    /:disease/table
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

        [ { id       : string  -- "US"
          , name     : string  -- "United States"
          , the      : boolean -- true
          }
        ]

    /data/region/current
        guess the current region by the sender's IP address

        string

    /data/region/:id
        get additional information about the given region

        { id       : string  -- "US"
        , name     : string  -- "United States"
        , the      : boolean -- true
        , geometry : object? -- { ... GeoJSON geometry ... }
        }

    /data/stats/:disease
        latest disease stats for each region

        [ { id         : string
          , population : number?
          , cases      : number?
          , deaths     : number?
          , recoveries : number?
          }
        ]

    /data/stats/:disease/in/:region
        disease stats over time within the given region

        [ { date       : string
          , population : number?
          , cases      : number?
          , deaths     : number?
          , recoveries : number?
          }
        ]
```
