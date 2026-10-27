# Song pictures

How `media/custom/songs/` was made (2026-10-27): `reference.jpg` is a style
sheet made in ChatGPT; `generate.py` asks OpenAI's image API (gpt-image-2,
medium quality, 1024×1024) to redraw each picture from it on its own, with no
label. Needs `OPENAI_API_KEY` in `~/.config/openai/env`. Outputs land next to
the script; each was checked by eye (BINGO, 5 Little Monkeys and 5 Little
Ducks needed firmer prompts; the ducks were cropped from a framed attempt),
then scaled to 512×512, the off-white background flattened and saved as a
96-colour PNG.

    python3 generate.py                     # all missing pictures
    python3 generate.py 06-bingo            # just one
