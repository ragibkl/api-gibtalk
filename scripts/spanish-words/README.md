# Spanish words for symbol search

Makes `media/words-es.yaml`: ARASAAC's own Spanish keywords (and plurals) for
each symbol, so "amigo" finds the same pictures as "friend". Added 2026-11-17
after Spanish searches from Madrid found nothing.

Our ARASAAC files are named after the pictogram's first English keyword, with
`_1`, `_2` for repeats, but not the pictogram id. So:

1. Download ARASAAC's keyword lists (slow, a few minutes each):

   ```sh
   mkdir -p work && cd work
   curl -o all-en.json https://api.arasaac.org/v1/pictograms/all/en
   curl -o all-es.json https://api.arasaac.org/v1/pictograms/all/es
   ```

2. For names shared by several pictograms (or several of our files), compare
   the images to pair each file with its pictogram. This downloads about 7,300
   small PNGs from static.arasaac.org into `work/img/` and writes
   `work/matched.json`.

   ```sh
   python3 build.py work ../../media --ambiguous
   npm install sharp
   node match.mjs work ../../media/arasaac-symbols
   ```

3. Write `media/words-es.yaml`:

   ```sh
   python3 build.py work ../../media
   ```

Symbols from the other libraries borrow the Spanish of the ARASAAC pictogram
with the same English name (the oldest one when several share it).
`tests/search-queries.yaml` has Spanish cases (`lang: es`).
