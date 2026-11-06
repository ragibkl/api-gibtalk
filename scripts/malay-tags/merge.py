#!/usr/bin/env python3
"""Merge out/*.json into media/words-ms.yaml, one symbol per line. Run from the repo root."""
import glob, json, pathlib

words = {}
for path in glob.glob("scripts/malay-tags/out/*.json"):
    words.update(json.load(open(path)))

lines = []
for file in sorted(words):
    unique = list(dict.fromkeys(w.strip().lower() for w in words[file] if w.strip()))
    lines.append(f"{json.dumps(file, ensure_ascii=False)}: {json.dumps(unique, ensure_ascii=False)}\n")

pathlib.Path("media/words-ms.yaml").write_text("".join(lines))
print(len(lines), "symbols")
