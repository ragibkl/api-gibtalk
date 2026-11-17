"""Build media/words-es.yaml: ARASAAC's Spanish keywords for our ARASAAC symbols.

usage: python3 build.py <dir with all-en.json, all-es.json, matched.json> <api-gibtalk/media>

Our files are named after the pictogram's first English keyword (_1, _2 for
repeats). A name used by one ARASAAC pictogram maps straight to it; for the
rest, matched.json (from match.mjs) pairs files with pictograms by comparing
the images. A poor image match falls back to the name, but only when every
pictogram with that name has the same Spanish words.

match.mjs reads ambiguous.json, made from all-en.json like this:

    groups = files grouped by name without _N; keep each group whose name is
    shared by more than one file or pictogram;
    json.dump([{"files": fs, "ids": by_name[name]} ...], "ambiguous.json")

Run `python3 build.py <dir> <media> --ambiguous` to write it.
"""
import collections, json, os, re, sys

src, media = sys.argv[1], sys.argv[2]
en = json.load(open(os.path.join(src, "all-en.json")))
es = {p["_id"]: p for p in json.load(open(os.path.join(src, "all-es.json")))}
matched = json.load(open(os.path.join(src, "matched.json"))) if "--ambiguous" not in sys.argv else {}
GOOD_MATCH = 50000  # summed squared grey difference over 24x24 pixels

by_name = collections.defaultdict(list)
for p in en:
    words = [k["keyword"].strip().lower() for k in p.get("keywords", []) if k.get("keyword")]
    if words:
        by_name[words[0]].append(p["_id"])

def spanish(pid):
    out = []
    for k in es.get(pid, {}).get("keywords", []):
        for w in (k.get("keyword"), k.get("plural")):
            w = (w or "").strip().lower()
            if w and w not in out:
                out.append(w)
    return out

files = sorted(os.listdir(os.path.join(media, "arasaac-symbols")))
base_of = lambda f: re.sub(r"_\d+$", "", f.rsplit(".", 1)[0]).lower()

if "--ambiguous" in sys.argv:
    groups = collections.defaultdict(list)
    for f in files:
        groups[base_of(f)].append(f)
    out = [{"files": fs, "ids": by_name[b]} for b, fs in groups.items()
           if by_name.get(b) and not (len(fs) == 1 and len(by_name[b]) == 1)]
    json.dump(out, open(os.path.join(src, "ambiguous.json"), "w"))
    sys.exit(f"wrote {len(out)} groups to ambiguous.json; now run match.mjs")

result, how = {}, collections.Counter()
for f in files:
    base = re.sub(r"_\d+$", "", f.rsplit(".", 1)[0]).lower()
    ids = by_name.get(base, [])
    words = []
    if len(ids) == 1 and sum(re.sub(r"_\d+$", "", g.rsplit(".", 1)[0]).lower() == base for g in files) == 1:
        words, kind = spanish(ids[0]), "name"
    elif f in matched and matched[f]["d"] <= GOOD_MATCH:
        words, kind = spanish(matched[f]["id"]), "image"
    elif ids and len({tuple(spanish(i)[:1]) for i in ids}) == 1:
        words, kind = spanish(ids[0]), "same-words"
    else:
        kind = "none"
    how[kind] += 1
    if words:
        result[f"arasaac-symbols/{f}"] = words

# Other libraries (Mulberry, Tawasol, our own) have no Spanish. A symbol whose
# name is an ARASAAC pictogram's English name borrows its Spanish words. When
# several pictograms share the name, the oldest (lowest id) is usually the
# plain meaning: "milk" → leche, not ordeñar (to milk).
def entries(path):
    # The library files are simple: "- name: x" then "  file: y".
    name = None
    for line in open(path, encoding="utf-8"):
        if line.startswith("- name:"):
            name = line.split(":", 1)[1].strip().strip('"')
        elif line.strip().startswith("file:") and name:
            yield {"name": name, "file": line.split(":", 1)[1].strip().strip('"')}
            name = None

for lib in sorted(os.listdir(media)):
    if not lib.endswith(".yaml") or lib.startswith(("arasaac", "words-")):
        continue
    for entry in entries(os.path.join(media, lib)):
        name = re.sub(r"-variant-\d+$", "", entry["name"]).replace("-", " ").lower()
        ids = by_name.get(name, [])
        if ids and spanish(min(ids)):
            result[f"{lib[:-5]}/{entry['file']}"] = spanish(min(ids))
            how["by-name"] += 1

with open(os.path.join(media, "words-es.yaml"), "w") as out:
    out.write("# ARASAAC's Spanish keywords (and plurals) for each ARASAAC symbol, by\n")
    out.write("# path under media. Pictograms by Sergio Palao, ARASAAC (https://arasaac.org),\n")
    out.write("# CC BY-NC-SA 4.0, Government of Aragón. Made by scripts/spanish-words.\n")
    for k, v in result.items():
        out.write(json.dumps(k, ensure_ascii=False) + ": " + json.dumps(v, ensure_ascii=False) + "\n")
print(dict(how), len(result))
