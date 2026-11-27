"""Restore the private FigCraft study using the owner's card collection (Pillow required)."""
import csv
import hashlib
import json
import os
import sys
from itertools import count
from pathlib import Path
from tempfile import TemporaryDirectory
from zipfile import ZIP_DEFLATED, ZipFile, ZipInfo

from PIL import Image


def publish_document(staged, destination):
    """Install the complete archive without replacing an existing directory entry."""
    contents = staged.read_bytes()
    for number in count():
        if number == 0:
            candidate = destination
        else:
            suffix = "restored" if number == 1 else f"restored-{number}"
            candidate = destination.with_stem(f"{destination.stem}.{suffix}")
        try:
            os.link(staged, candidate)
        except FileExistsError:
            if candidate.is_file() and candidate.read_bytes() == contents:
                print(f"Already restored: {candidate}")
                return
        else:
            print(f"Restored: {candidate}")
            return


here = Path(__file__).resolve().parent
root = here.parents[1]
collection = Path(sys.argv[1]).expanduser()
private = root / "local" / "design-round"
private.mkdir(exist_ok=True, parents=True)
with (root / "docs/foundation/card-content.csv").open() as f:
    identities = {r["identity"]: int(r["tts_id"]) for r in csv.DictReader(f) if r["tts_id"]}
faces = {r["card_id"]: collection / r["face"] for r in json.loads((collection / "catalog/cards.json").read_text())}

# Stage on the destination filesystem for atomic publication. Fixed ZIP metadata
# makes equivalent rebuilds byte-identical even when the thumbnails were regenerated.
with (
    TemporaryDirectory(prefix=".restore-", dir=here) as work,
    TemporaryDirectory(dir=private) as thumbnail_dir,
):
    staging = Path(work)
    thumbnails = Path(thumbnail_dir)
    assets = list(here.glob("park-*.png"))
    for serial in [1, 2, 4, 5, 8, 11, 14, *range(61, 68)]:
        identity = ("FA" if serial >= 61 else "ST") + f"-{serial:03}"
        path = thumbnails / (identity + ".jpg")
        with Image.open(faces[identities[identity]]) as face:
            thumb = face.convert("RGB")
            thumb.thumbnail((270, 376))
            thumb.save(path, quality=89)
        assets.append(path)
    by_hash = {hashlib.sha256(p.read_bytes()).hexdigest(): p for p in assets}
    for layout_name, document_name in (
        ("layout.json", "manager-decisions.figcraft"),
        ("combined-layout.json", "combined.figcraft"),
    ):
        layout = (here / layout_name).read_bytes()
        staged = staging / document_name
        with ZipFile(staged, "w") as archive:
            archive.writestr(ZipInfo("document.json"), layout, ZIP_DEFLATED)
            for item in json.loads(layout)["imageIndex"]:
                digest = item["hash"]
                archive.writestr(ZipInfo("images/" + digest), by_hash[digest].read_bytes(), ZIP_DEFLATED)
    for thumbnail in thumbnails.glob("*.jpg"):
        os.replace(thumbnail, private / thumbnail.name)
    for document in staging.glob("*.figcraft"):
        publish_document(document, here / document.name)
