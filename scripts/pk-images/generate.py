"""Generates the Permata Kurnia board pictures the symbol libraries don't have.

Asks OpenAI's image API (gpt-image-2) to draw each one in the style of
reference.png (library pictograms, built by make_reference.py). Needs
OPENAI_API_KEY in ~/.config/openai/env. Outputs land next to this script as
<slug>-a.png / <slug>-b.png (two candidates); pick one by eye.

    python3 generate.py                # all missing pictures
    python3 generate.py lion-dance     # just one
"""
import base64, json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

D = os.path.dirname(os.path.abspath(__file__))
KEY = [l.split("=", 1)[1].strip() for l in open(os.path.expanduser("~/.config/openai/env"))
       if l.startswith("OPENAI_API_KEY=")][0]

PICTURES = [
    ("gong-xi-fa-cai", "a Chinese boy and girl in red Chinese New Year clothes, smiling and making the fist-in-palm greeting gesture"),
    ("new-clothes", "a child proudly wearing a new bright red Chinese-style outfit with gold buttons, with a small sparkle showing it is new"),
    ("red-packet", "two red Chinese New Year money envelopes (angpau) with gold decoration, no writing"),
    ("lou-sang", "a large round plate of yee sang (Chinese New Year raw fish salad) with colourful shredded vegetables, and chopsticks tossing it up"),
    ("cny-biscuits", "Chinese New Year cookies: round pineapple tarts with yellow jam centres, rolled love-letter wafers and small peanut cookies"),
    ("chinese-lantern", "three red round Chinese paper lanterns with gold tops and tassels, hanging in a row"),
    ("lion-dance", "a Chinese lion dance lion costume, red and gold with big eyes and fluffy trim, dancing with two people's legs visible underneath"),
    ("firecracker", "a string of red Chinese firecrackers hanging down, with a small burst of sparks at the bottom, no writing"),
    ("uncle", "head and shoulders of a friendly middle-aged man with black hair and a moustache, in a red jumper"),
    ("aunty", "head and shoulders of a friendly middle-aged woman with long black curly hair and glasses, in a purple top"),
    ("fidget-toy", "two sensory fidget toys side by side: a red three-armed fidget spinner and an orange squishy spiky stress ball"),
    ("doa", "two hands held together and cupped open upwards, palms facing up, as in Muslim prayer of supplication (doa)"),
    ("pressure", "an adult leaning over a small worried child, pointing and pushing them to hurry, with a few short stress lines around the child"),
    ("by-myself", "a proud child pointing to their own chest with one hand while stacking blocks with the other, an adult's hand held back at the side"),
    ("too-close", "two children standing very close face to face, one leaning back with a hand up and a small red double arrow showing they want more space"),
    ("sensory-overload", "a child covering both ears with eyes squeezed shut, surrounded by small zigzag noise lines, bright light rays and a busy swirl"),
    ("steps", "three numbered steps in a row going up like stairs, labelled 1, 2 and 3, with a small arrow from one to the next (the numbers 1, 2, 3 MUST appear)"),
    ("not-ready", "a child shaking their head with one hand raised in a 'not yet' gesture, beside a small clock"),
    ("do", "a simple stick figure busy doing an activity with both hands, stacking a block onto a small tower, with a small curved motion arrow"),
    ("different", "a red circle and a blue square side by side with a large black not-equal sign between them"),
    ("excited", "a very excited happy child jumping with both fists raised in the air and a big open-mouth smile"),
]

def prompt(desc):
    return ("The attached sheet shows picture symbols from a children's AAC (talking picture board) symbol library. "
            f"Draw a new symbol in exactly the same style: {desc}. "
            "Same flat cartoon style, thick clean dark outlines, simple flat colours, plain white background. "
            "Square, centred, with comfortable empty space around the edges. A single picture only: "
            "no words, letters, numbers, label, border or frame.")

def gen(job):
    (slug, desc), suffix = job
    out = f"{D}/{slug}-{suffix}.png"
    if os.path.exists(out):
        return slug + suffix, "exists"
    r = subprocess.run(["curl", "-s", "-m", "300", "https://api.openai.com/v1/images/edits",
                        "-H", f"Authorization: Bearer {KEY}",
                        "-F", "model=gpt-image-2", "-F", f"image[]=@{D}/reference.png",
                        "-F", f"prompt={prompt(desc)}", "-F", "size=1024x1024",
                        "-F", "quality=medium", "-F", "n=1"], capture_output=True, text=True)
    try:
        d = json.loads(r.stdout)
        if "error" in d:
            return slug + suffix, "ERROR " + d["error"].get("message", "")[:120]
        open(out, "wb").write(base64.b64decode(d["data"][0]["b64_json"]))
        return slug + suffix, "ok"
    except Exception as e:
        return slug + suffix, f"FAIL {e} {r.stdout[:120]}"

if __name__ == "__main__":
    only = sys.argv[1:]
    jobs = [(p, s) for p in PICTURES if not only or p[0] in only for s in "ab"]
    with ThreadPoolExecutor(6) as ex:
        for name, status in ex.map(gen, jobs):
            print(name, status, flush=True)
