"""Restore the private FigCraft study using the owner's card collection (Pillow required)."""
import csv
import hashlib
import json
import sys
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

from PIL import Image

here = Path(__file__).resolve().parent
root = here.parents[1]
collection = Path(sys.argv[1]).expanduser()
private = root / "local" / "design-round"
private.mkdir(exist_ok=True, parents=True)
with (root / "docs/foundation/card-content.csv").open() as f:
    identities = {r["identity"]: int(r["tts_id"]) for r in csv.DictReader(f) if r["tts_id"]}
faces = {r["card_id"]: collection / r["face"] for r in json.loads((collection / "catalog/cards.json").read_text())}
assets = list(here.glob("park-*.png"))
for serial in [1, 2, 4, 5, 8, 11, 14, *range(61, 68)]:
    identity = ("FA" if serial >= 61 else "ST") + f"-{serial:03}"
    with Image.open(faces[identities[identity]]) as face:
        thumb = face.convert("RGB")
        thumb.thumbnail((270, 376))
        path = private / (identity + ".jpg")
        thumb.save(path, quality=89)
        assets.append(path)
by_hash = {hashlib.sha256(p.read_bytes()).hexdigest(): p for p in assets}
layout = (here / "layout.json").read_bytes()
images = [by_hash[item["hash"]] for item in json.loads(layout)["imageIndex"]]
with ZipFile(here / "manager-decisions.figcraft", "w", ZIP_DEFLATED) as z:
    z.writestr("document.json", layout)
    for image in images:
        z.write(image, "images/" + hashlib.sha256(image.read_bytes()).hexdigest())
