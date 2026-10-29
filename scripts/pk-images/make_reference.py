"""Builds reference.png: a sheet of library pictograms used as the style guide."""
import os
from PIL import Image
D = os.path.dirname(os.path.abspath(__file__))
M = os.path.join(D, "..", "..", "media", "arasaac-symbols")
FILES = ["grandfather_3.png", "grandmother_2.png", "mother.png", "father.png",
         "hello.png", "firecracker.png", "mandarin.png", "money.png", "sing.png"]
sheet = Image.new("RGB", (900, 900), "white")
for i, f in enumerate(FILES):
    im = Image.open(os.path.join(M, f)).convert("RGBA")
    im.thumbnail((270, 270))
    bg = Image.new("RGBA", im.size, "white")
    bg.alpha_composite(im)
    sheet.paste(bg.convert("RGB"), (15 + (i % 3) * 300, 15 + (i // 3) * 300))
sheet.save(os.path.join(D, "reference.png"))
