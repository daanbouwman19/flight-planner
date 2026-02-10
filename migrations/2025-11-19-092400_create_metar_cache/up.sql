CREATE TABLE metar_cache (
    station TEXT PRIMARY KEY NOT NULL,
    raw TEXT NOT NULL,
    flight_rules TEXT,
    observation_time TEXT,
    observation_dt TEXT,
    fetched_at TEXT NOT NULL
);
