import base64, json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor
D=os.path.dirname(os.path.abspath(__file__))
KEY=[l.split("=",1)[1].strip() for l in open(os.path.expanduser("~/.config/openai/env")) if l.startswith("OPENAI_API_KEY=")][0]
SONGS=[
 ("01-wheels-on-the-bus","Wheels on the Bus","a yellow school bus seen from the side, with a circular arrow near one wheel"),
 ("02-if-youre-happy","If You're Happy","a big smiling face next to two clapping hands"),
 ("04-old-macdonald","Old MacDonald","a farmer in blue overalls and a straw hat, a green tractor and a black-and-white cow"),
 ("05-twinkle-twinkle","Twinkle Twinkle Little Star","a glowing yellow star in a dark blue night sky with tiny stars (the whole background is the night sky)"),
 ("06-bingo","BINGO","a white puppy with brown spots and a red collar, beside five colourful toy blocks. Each block clearly shows one big capital letter, together spelling B, I, N, G, O. The letters B I N G O MUST appear on the blocks"),
 ("07-mary-had-a-little-lamb","Mary Had a Little Lamb","a little girl with pigtails in a pink top and purple skirt, walking beside a white lamb"),
 ("08-baby-shark","Baby Shark","a friendly grey baby shark swimming in light blue water with bubbles (the whole background is water)"),
 ("09-head-and-shoulders","Head and Shoulders","a boy in a blue t-shirt touching his head, with small arrows pointing to his head, shoulders and knees"),
 ("10-five-little-monkeys","5 Little Monkeys","a large blue numeral 5 on the left, MUST be included, and little brown monkeys jumping on a bed on the right"),
 ("11-five-little-ducks","5 Little Ducks","a large blue numeral 5 on the left, MUST be included, and exactly five (5, not six) yellow ducklings on a small pond on the right"),
 ("12-this-is-the-way","This is the Way","a toothbrush with toothpaste, a blue arrow, and two hands being washed with soap bubbles"),
 ("13-row-your-boat","Row Row Row Your Boat","a child rowing a small brown wooden boat on blue water"),
 ("14-ring-around-the-rosie","Ring Around the Rosie","four children holding hands in a line as if dancing in a circle, with a curved blue arrow above them"),
 ("15-baa-baa-black-sheep","Baa Baa Black Sheep","a black sheep next to three sacks full of wool"),
 ("16-bangun-pagi","Bangun Pagi (Wake Up in the Morning)","a child waking up and stretching in a bed with a blue blanket, an alarm clock and a rising sun"),
 ("17-family-finger","Family Finger","an open hand with a small smiling family member's face on each fingertip"),
 ("18-one-little-finger","One Little Finger","a large blue number 1 next to a hand holding up one finger (the number is allowed)"),
 ("19-rain-rain-go-away","Rain Rain Go Away","a grey rain cloud with raindrops and the sun peeking out, inside a red 'no' circle with a diagonal line"),
 ("20-do-re-mi","Do Re Mi","colourful music notes above a staircase of rainbow-coloured steps"),
]
def prompt(title, desc):
    return (f'The attached sheet is a set of picture symbols for a children\'s AAC (talking picture board) app. '
            f'Redraw only the "{title}" picture ({desc}) as a single standalone image. '
            'Keep exactly the same drawing style, line weight and colours as the sheet: flat cartoon, clean dark outlines, plain off-white background unless the description says otherwise. '
            'Square, centred, with comfortable empty space around the edges. Do not add a label, caption or any other words; only include letters or numbers that the description asks for. No border or frame.')
def gen(item, suffix=""):
    slug,title,desc=item
    out=f"{D}/{slug}{suffix}.png"
    if os.path.exists(out): return slug,"exists"
    r=subprocess.run(["curl","-s","-m","300","https://api.openai.com/v1/images/edits","-H",f"Authorization: Bearer {KEY}",
        "-F","model=gpt-image-2","-F",f"image[]=@{D}/reference.jpg","-F",f"prompt={prompt(title,desc)}",
        "-F","size=1024x1024","-F","quality=medium","-F","n=1"],capture_output=True,text=True)
    try:
        d=json.loads(r.stdout)
        if "error" in d: return slug,"ERROR "+d["error"].get("message","")[:120]
        open(out,"wb").write(base64.b64decode(d["data"][0]["b64_json"]))
        return slug,"ok"
    except Exception as e:
        return slug,f"FAIL {e} {r.stdout[:120]}"
if __name__=="__main__":
    only=sys.argv[1:]
    items=[s for s in SONGS if not only or s[0] in only]
    with ThreadPoolExecutor(4) as ex:
        for slug,status in ex.map(gen, items): print(slug,status,flush=True)
