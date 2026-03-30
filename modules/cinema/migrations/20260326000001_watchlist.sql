CREATE TABLE IF NOT EXISTS collections (
    id INTEGER PRIMARY KEY,
    collection TEXT NOT NULL,
    media_type TEXT NOT NULL,
    tmdb_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    poster_path TEXT,
    added_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    UNIQUE(collection, media_type, tmdb_id)
);
