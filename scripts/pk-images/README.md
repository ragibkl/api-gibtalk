# Permata Kurnia and self-advocacy board pictures

How `media/custom/pk/` was made (2026-10-29): a few tiles on Permata Kurnia's
communication boards (feelings, sick, Chinese New Year, assembly, canteen,
classroom) had no match in the symbol libraries. `make_reference.py` builds
`reference.png` from library pictograms as a style guide; `generate.py` asks
OpenAI's image API (gpt-image-2, medium quality, 1024×1024) for two
candidates of each picture. Needs `OPENAI_API_KEY` in `~/.config/openai/env`.
The better candidate was picked by eye, scaled to 512×512, near-white
flattened to white and saved as a 96-colour PNG.

    python3 make_reference.py               # needs Pillow
    python3 generate.py                     # all missing pictures
    python3 generate.py lion-dance          # just one
