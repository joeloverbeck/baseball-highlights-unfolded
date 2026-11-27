"""Exercise the restore command in a disposable checkout with a generated face."""
import csv
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from zipfile import ZipFile

from PIL import Image

ROOT = Path(__file__).resolve().parents[1]


class RestoreStudy(unittest.TestCase):
    def test_repeat_restore_preserves_edits(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            art = root / "art/manager-review"
            art.mkdir(parents=True)
            shutil.copy(ROOT / "art/manager-review/restore.py", art)
            foundation = root / "docs/foundation"
            foundation.mkdir(parents=True)
            ledger = foundation / "card-content.csv"
            shutil.copy(ROOT / "docs/foundation/card-content.csv", ledger)
            collection = root / "collection"
            catalog = collection / "catalog"
            catalog.mkdir(parents=True)
            with ledger.open(newline="") as source:
                records = [{"card_id": int(row["tts_id"]), "face": "test.png"}
                           for row in csv.DictReader(source) if row["tts_id"]]
            (catalog / "cards.json").write_text(json.dumps(records))
            Image.new("RGB", (16, 20), "green").save(collection / "test.png")
            park = art / "park-test.png"
            shutil.copy(collection / "test.png", park)
            digest = hashlib.sha256(park.read_bytes()).hexdigest()
            layout = json.dumps({"imageIndex": [{"hash": digest}]})
            for name in ("layout.json", "combined-layout.json"):
                (art / name).write_text(layout)

            def restore():
                subprocess.run([sys.executable, art / "restore.py", collection],
                               check=True, capture_output=True)

            restore()
            documents = [art / name for name in ("combined.figcraft", "manager-decisions.figcraft")]
            for document in documents:
                with ZipFile(document) as archive:
                    self.assertEqual(archive.read("document.json"), layout.encode())
                    self.assertEqual(archive.read("images/" + digest), park.read_bytes())
            baseline = documents[0].read_bytes()
            restore()
            self.assertEqual(set(art.glob("*.figcraft")), set(documents))
            for document in documents:
                with ZipFile(document, "a") as archive:
                    archive.writestr("local-edit", "my edit")
            edited = documents[0].read_bytes()
            restore()
            for document in documents:
                with ZipFile(document) as archive:
                    self.assertEqual(archive.read("local-edit"), b"my edit")
                self.assertEqual(document.with_suffix(".restored.figcraft").read_bytes(), baseline)
            self.assertEqual(documents[0].read_bytes(), edited)
            restore()
            self.assertEqual(len(list(art.glob("*.figcraft"))), 4)
            companion = art / "combined.restored.figcraft"
            companion.write_bytes(b"second local edit")
            restore()
            self.assertEqual(companion.read_bytes(), b"second local edit")
            self.assertEqual((art / "combined.restored-2.figcraft").read_bytes(), baseline)
