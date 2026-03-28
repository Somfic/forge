# forge

## Docker

```sh
docker run -e FORGE_TMDB_API_KEY=your_api_key -v ./data:/app/data -p 3000:3000 -p 6881:6881 ghcr.io/somfic/forge
```

The `data/` directory must be persisted across container restarts using a volume mount (`-v`). It contains media and application state.

## infra

- [ ] fail2ban

## modules

- [ ] photos / videos
- [ ] movies / tv
    - [x] tmdb integration
    - [ ] auto skip intro
    - [x] subtitles
    - [ ] watch parties
        - [ ] sync playback
- [ ] music
- [ ] books
