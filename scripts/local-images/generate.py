"""Generates Malaysian symbols the libraries don't have (media/custom/local).

Asks OpenAI's image API (gpt-image-2) to draw each one in the style of
../pk-images/reference.png, two candidates each; pick one by eye. Needs
OPENAI_API_KEY in ~/.config/openai/env.

    python3 generate.py <out dir>            # all missing pictures
    python3 generate.py <out dir> durian     # just one
"""
import base64, json, os, subprocess, sys
from concurrent.futures import ThreadPoolExecutor

D = os.path.dirname(os.path.abspath(__file__))
REFERENCE = f"{D}/../pk-images/reference.png"
KEY = [l.split("=", 1)[1].strip() for l in open(os.path.expanduser("~/.config/openai/env"))
       if l.startswith("OPENAI_API_KEY=")][0]

PICTURES = [
    # Nasi lemak
    ("nasi-lemak", "a plate of Malaysian nasi lemak on a banana leaf: a round mound of white coconut rice, red sambal, a half boiled egg, fried anchovies and peanuts, and cucumber slices"),
    ("nasi-lemak-bungkus", "a Malaysian nasi lemak bungkus: a small pyramid-shaped takeaway packet wrapped in brown paper over a green banana leaf, folded neatly at the top, sitting next to an opened packet showing white rice, red sambal, a slice of egg and fried anchovies inside"),
    # Daily food
    ("roti-canai", "Malaysian roti canai: a round golden flaky flatbread on a plate, with a small bowl of dhal curry beside it"),
    ("teh-tarik", "a Malaysian teh tarik: a person's hands pouring frothy milk tea in a long stream from one metal mug down into another"),
    ("teh-tarik-mug", "a clear glass mug with a handle, full of light brown milk tea with a thick pale foam on top, as served in a Malaysian mamak restaurant"),
    ("nasi-goreng", "a plate of Malaysian nasi goreng: brown-red fried rice with bits of egg and vegetables, a fried egg on top, cucumber slices at the side"),
    ("mee-goreng", "a plate of Malaysian mee goreng mamak: reddish fried yellow noodles with tofu, egg and green vegetables, a lime half at the side"),
    ("laksa", "a bowl of Malaysian curry laksa: thick noodles in orange coconut curry soup, with tofu puffs, prawns and half a boiled egg"),
    ("sambal", "a small bowl of red Malaysian sambal chilli paste with a spoon, and two red chillies beside it"),
    ("kuih", "a plate of assorted Malay kuih: a green and white layered square, a pink and white layered square, and green round onde-onde balls rolled in white coconut"),
    ("sirap", "a tall glass of bright red-pink rose syrup drink with ice cubes and a straw"),
    ("tosai", "South Indian tosai: a large thin golden crepe folded on a banana leaf, with small bowls of white coconut chutney and orange sambar"),
    # Fruit and treats
    ("durian", "a whole spiky green durian fruit next to a split open durian showing its pale yellow flesh"),
    ("rambutan", "a small bunch of hairy red rambutan fruits, one peeled open showing the white flesh"),
    ("manggis", "a purple mangosteen fruit with green top next to one cut open showing its white segments"),
    ("cendol", "a bowl of Malaysian cendol: shaved ice with green jelly strands, red beans, coconut milk and brown palm sugar syrup"),
    ("ais-kacang", "a bowl of Malaysian ais kacang: a tall mound of shaved ice with red and green syrup, sweet corn, red beans and jelly cubes"),
    ("rendang", "a plate of Malaysian beef rendang: dark brown chunks of beef in thick spicy coconut sauce, on a banana leaf"),
    ("satay", "several Malaysian satay sticks: grilled meat on bamboo skewers, with a small bowl of peanut sauce and cucumber pieces"),
    # Home and toilet
    ("tandas-cangkung", "an Asian white ceramic squat toilet set in a tiled floor, seen from above at an angle, with a water hose on the wall"),
    ("gayung", "a plastic water dipper with a long handle, called a gayung, blue, next to a small bucket of water"),
    ("kain-batik", "a folded Malaysian batik sarong cloth with colourful flower and leaf patterns"),
    ("kain-pelikat", "a Malaysian man's checked sarong, kain pelikat, folded, in dark green and blue check"),
    ("songkok", "head and shoulders of a smiling Malay boy wearing a black velvet songkok, a tall oval boat-shaped cap, in a white shirt"),
    ("baju-kurung", "a Malay girl wearing a pink baju kurung: long loose top and long matching skirt with a small flower pattern"),
    ("baju-melayu", "a Malay boy wearing a blue baju melayu: loose long-sleeved top and matching trousers, with a short patterned sampin cloth at the waist and a black songkok"),
    # Faith and festivals
    ("wuduk", "a Malay boy with a white cap and rolled up sleeves washing his forearm under a water tap fixed on a tiled wall, at a Malaysian surau ablution area: no sink or basin, the water falls straight down to a tiled floor channel with a drain"),
    ("sahur", "a Malay family eating together at a table before dawn, the window behind them showing a dark sky with a crescent moon and stars"),
    ("deepavali", "Deepavali: three small lit clay oil lamps in front of a colourful round kolam floor pattern"),
    ("hari-merdeka", "Malaysian children waving small Malaysian flags and cheering on Independence Day"),
    ("aidiladha", "a Muslim family in clean Eid clothes standing beside a healthy goat and a cow in front of a mosque, peaceful and happy"),
    ("gawai", "an Iban man and woman in traditional Sarawak Gawai festival costume, with hornbill feather headdress and silver jewellery, dancing"),
    ("thaipusam", "a Malaysian Hindu devotee in a yellow dhoti walking and carrying a brass milk pot decorated with flowers on the head, for Thaipusam"),
    ("wesak", "a pink lotus flower with lit candles around it and a small hanging lantern, for Wesak Day"),
    ("duit-raya", "two small green Hari Raya money packets with gold pattern, with Malaysian ringgit notes peeking out"),
    # Deepavali
    ("kolam", "a colourful round kolam floor pattern made of coloured rice powder, with flower and petal shapes"),
    ("pelita", "a single small lit clay oil lamp, a diya, with a bright flame"),
    ("sari", "a Malaysian Indian woman wearing a bright pink and gold sari, with a small red pottu on her forehead"),
    ("kurta", "a Malaysian Indian boy wearing a cream kurta top with gold trim and matching trousers"),
    ("murukku", "a plate of murukku: golden crunchy spiral-shaped Indian snacks"),
    ("ladoo", "a plate of Indian sweets: round orange ladoo balls and small square diamond-shaped sweets"),
    ("inai", "a hand with a beautiful brown henna flower pattern on the palm and fingers"),
    ("kalungan-bunga", "a garland of orange marigolds and white jasmine flowers"),
    ("kuil-hindu", "a South Indian Hindu temple in Malaysia with a tall colourful tiered gopuram tower, no figures shown in detail"),
    ("mandi-minyak", "a Malaysian Indian boy in a towel at home having warm sesame oil gently put on his hair by his mother, the Deepavali morning oil bath, a small bowl of oil beside them"),
    ("pottu", "the face of a smiling Malaysian Indian girl with a red pottu dot on her forehead"),
    ("selamat-deepavali", "a Malaysian Indian family in sari and kurta smiling and greeting with palms pressed together"),
    ("nasi-daun-pisang", "Malaysian banana leaf rice: white rice on a big banana leaf with small piles of vegetable curry, a papadum and pickles"),
    # School and places
    ("school-uniform-boy", "a Malaysian primary school boy in uniform: short-sleeved white shirt, navy blue shorts, white socks and white canvas shoes"),
    ("school-uniform-girl", "a Malaysian primary school girl in uniform: white baju kurung top, long navy blue skirt, white tudung headscarf and white canvas shoes"),
    ("pengawas", "a Malaysian school prefect: a teenage student in a dark blazer with a badge and a tie, standing straight holding a small notebook"),
    ("pasar-malam", "a Malaysian night market: rows of stalls with coloured canopies and hanging light bulbs under a dark evening sky, people buying food"),
    ("kedai-runcit", "a small Malaysian neighbourhood sundry shop: open shop front with shelves of tins and packets, sacks of rice and hanging snacks, blank signboard"),
    ("mamak", "a Malaysian mamak restaurant: open-air shop with plastic tables and stools, a man in a white cap stretching roti at a stall, blank signboard"),
    ("rumah-kampung", "a traditional Malay wooden kampung house on stilts with a steep roof, wooden stairs in front and a coconut tree beside it"),
    # Games and animals
    ("congkak", "a Malaysian congkak game: a long wooden boat-shaped board with two rows of round holes filled with small shells or marbles"),
    ("sepak-takraw", "a boy kicking a woven rattan sepak takraw ball high over a net with his foot"),
    ("batu-seremban", "a child's hand tossing small cloth bean bags in the air, with more small bags on the floor, the game batu seremban"),
    ("wau-bulan", "a traditional Malaysian wau bulan moon kite: large crescent-shaped tail and wings with bright flower patterns, flying"),
    ("silat", "a Malay boy in black silat clothes with a short patterned sampin and a tanjak head cloth, in a low silat stance with open hands"),
    ("orang-utan", "an orange-haired orang utan sitting in a tree holding a branch with its long arm"),
    ("tapir", "a Malayan tapir: black front and legs, white middle, short flexible snout, standing"),
]

def prompt(desc):
    return ("The attached sheet shows picture symbols from a children's AAC (talking picture board) symbol library. "
            f"Draw a new symbol in exactly the same style: {desc}. "
            "Same flat cartoon style, thick clean dark outlines, simple flat colours, plain white background. "
            "Square, centred, with comfortable empty space around the edges. A single picture only: "
            "no words, letters, numbers, label, border or frame.")

def gen(job):
    (slug, desc), suffix = job
    path = f"{OUT}/{slug}-{suffix}.png"
    if os.path.exists(path):
        return slug + suffix, "exists"
    r = subprocess.run(["curl", "-s", "-m", "300", "https://api.openai.com/v1/images/edits",
                        "-H", f"Authorization: Bearer {KEY}",
                        "-F", "model=gpt-image-2", "-F", f"image[]=@{REFERENCE}",
                        "-F", f"prompt={prompt(desc)}", "-F", "size=1024x1024",
                        "-F", "quality=medium", "-F", "n=1"], capture_output=True, text=True)
    try:
        d = json.loads(r.stdout)
        if "error" in d:
            return slug + suffix, "ERROR " + d["error"].get("message", "")[:120]
        open(path, "wb").write(base64.b64decode(d["data"][0]["b64_json"]))
        return slug + suffix, "ok"
    except Exception as e:
        return slug + suffix, f"FAIL {e} {r.stdout[:120]}"

if __name__ == "__main__":
    OUT, only = sys.argv[1], sys.argv[2:]
    os.makedirs(OUT, exist_ok=True)
    jobs = [(p, s) for p in PICTURES if not only or p[0] in only for s in "ab"]
    with ThreadPoolExecutor(6) as ex:
        for name, status in ex.map(gen, jobs):
            print(name, status, flush=True)
