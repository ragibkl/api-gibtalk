# api-gibtalk

REST API backend for the [GibTalk AAC (Augmentative and Alternative Communication) App](https://gibtalk.com). Serves searchable symbol/icon assets used in AAC communication tools.

**Live at:** https://api.gibtalk.com/

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
curl "https://api.gibtalk.com/api/symbols/search/?q=apple"
```

```json
[
  { "url": "https://api.gibtalk.com/media/arasaac-symbols/apple.png" },
  { "url": "https://api.gibtalk.com/media/mulberry-symbols/apple.png" }
]
```

### Serve Symbol Images

```
GET /media/{library}/{filename}.png
```

Static file serving for symbol images.

### Templates

```
GET /api/templates
GET /api/templates/{id}
```

Ready-made word sets the app offers. The list gives each template's id, name,
description, author, main language, word count and URL; one template adds its
words (label, language, picture URL, children for folders).

```bash
curl "https://api.gibtalk.com/api/templates"
```

```json
[
  {
    "id": "greetings-ms",
    "name": "Ucapan",
    "description": "Basic Greetings in Bahasa Malaysia",
    "author": "GK",
    "language": "ms",
    "words": 21,
    "url": "https://api.gibtalk.com/api/templates/greetings-ms"
  }
]
```

The templates live in `templates/` as YAML, the same format as GibTalk's backup
files, listed in `templates/index.yaml`. Picture paths are relative to `media/`
(e.g. `arasaac-symbols/apple.png`), and the API turns them into full URLs with
`BASE_URL`, so moving hosts never means editing templates. The API won't start
if a template names a picture that isn't in `media/`.

## Symbol Libraries

| Library | Files |
|---------|-------|
| [ARASAAC](https://arasaac.org/) | ~11,700 |
| [Mulberry](https://mulberrysymbols.org/) | ~3,400 |
| [Tawasol](https://www.tawasolsymbols.org/) | ~1,600 |
| Custom | 46 |

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

## Licence

The code in this repository is under the [MIT licence](LICENSE).

The symbol images in `media/` are not: each library keeps its own licence.
ARASAAC symbols are by Sergio Palao, property of the Government of Aragón, under
[CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/); Mulberry
symbols are under [CC BY-SA 2.0 UK](https://creativecommons.org/licenses/by-sa/2.0/uk/);
Tawasol symbols are under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
Pictures in `media/custom/` were made for GibTalk and are free to use. The
`songs/` set was generated with OpenAI's image model (gpt-image-2, October 2026)
from a style sheet; `scripts/songs-images/` has the prompts and script. The
`pk/` set (pictures for the Permata Kurnia and self-advocacy boards that the libraries lack) was
made the same way, in the style of the library pictograms; see
`scripts/pk-images/`.
See [gibtalk.com/symbols](https://gibtalk.com/symbols/#credits) for full credits.
