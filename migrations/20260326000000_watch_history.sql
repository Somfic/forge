CREATE TABLE IF NOT EXISTS watch_history (
    id INTEGER PRIMARY KEY,
    media_type TEXT NOT NULL,
    tmdb_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    poster_path TEXT,
    season INTEGER NOT NULL DEFAULT 0,
    episode INTEGER NOT NULL DEFAULT 0,
    info_hash TEXT,
    file_idx INTEGER NOT NULL DEFAULT 0,
    progress REAL NOT NULL DEFAULT 0,
    duration REAL NOT NULL DEFAULT 0,
    last_watched TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    UNIQUE(media_type, tmdb_id)
);
