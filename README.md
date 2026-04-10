# api-gibtalk

REST API backend for the [GibTalk AAC (Augmentative and Alternative Communication) App](https://gibtalk.ragib.my). Serves searchable symbol/icon assets used in AAC communication tools.

**Live at:** https://api.gibtalk.ragib.my/

## Tech Stack

- **Rust** with [Axum](https://github.com/tokio-rs/axum) web framework and Tokio async runtime
- **Docker** multi-stage build, published to Docker Hub as `ragibkl/api-gibtalk`
- **GitHub Actions** CI/CD, triggered on version bumps to `master`

## API

### Search Symbols

```
GET /api/symbols/search/?q={query}
```

Searches symbol filenames across all libraries. Returns up to 100 results sorted by similarity.

**Example:**

```bash
curl "https://api.gibtalk.ragib.my/api/symbols/search/?q=apple"
```

```json
[
  { "url": "https://api.gibtalk.ragib.my/media/arasaac-symbols/apple.png" },
  { "url": "https://api.gibtalk.ragib.my/media/mulberry-symbols/apple.png" }
]
```

### Serve Symbol Images

```
GET /media/{library}/{filename}.png
```

Static file serving for symbol images.

## Symbol Libraries

| Library | Files |
|---------|-------|
| [ARASAAC](https://arasaac.org/) | ~11,700 |
| [Mulberry](https://mulberrysymbols.org/) | ~3,400 |
| [Tawasol](https://www.tawasolsymbols.org/) | ~1,600 |
| Custom | 5 |

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `BASE_URL` | `http://localhost:3000/` | Base URL prefix for symbol URLs in search results |

The server listens on port `3000`.

## Development

```bash
# Run locally
cargo run

# Build release
cargo build --release
```

## Docker

```bash
# Build
./scripts/build.sh

# Push to Docker Hub
./scripts/push.sh
```

The version tag is read from the `version` file.
