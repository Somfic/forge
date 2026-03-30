# forge

## Docker

```sh
docker run -e FORGE_TMDB_API_KEY=your_api_key -v ./data:/app/data -p 3000:3000 -p 6881:6881 ghcr.io/somfic/forge
```

### Environment variables

| Variable | Description | Default |
|---|---|---|
| `FORGE_HOST` | Bind address | `0.0.0.0` |
| `FORGE_PORT` | HTTP port | `3000` |
| `FORGE_DATA_DIR` | Data directory path | `./data/` |
| `FORGE_DATABASE_URL` | Database connection string (sqlite/postgres/mysql) | SQLite in data dir |
| `FORGE_CONFIG` | Config file path | `forge.toml` |
| `FORGE_TMDB_API_KEY` | TMDB API key (required) | |
| `FORGE_STREAM_SOURCES` | Comma-separated stream source URLs | `https://torrentio.strem.fun` |
| `FORGE_SUBTITLE_LANGUAGES` | Comma-separated subtitle languages | `en` |
| `FORGE_MAX_CONCURRENT_DOWNLOADS` | Max concurrent background downloads | `2` |
| `FORGE_TORRENT_PORT` | Torrent listen port | `6881` |
| `FORGE_USE_DHT` | Enable DHT for peer discovery | `true` |

## infra

- [ ] fail2ban

## modules

- [ ] photos / videos
- [ ] movies / tv
    - [x] tmdb integration
    - [ ] recommendations (able to turn off)
    - [ ] auto skip intro
    - [x] subtitles
        - [ ] configurable font + color
    - [ ] watch parties
        - [ ] sync playback
    - [ ] frame generation (lsfg) ??
    - [ ] external video player support (mpv, vlc, etc.)
- [ ] music
- [ ] books
