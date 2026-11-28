"""Stage private PhotoCraft exports for the current desktop game."""
import argparse
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory


def check_identities(label, expected, actual):
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing or extra:
        raise ValueError(f"{label} — missing: {', '.join(missing) or 'none'}; "
                         f"extra: {', '.join(extra) or 'none'}")


def stage(mode, source, review, exports, output):
    required = subprocess.run(
        ["cargo", "run", "--quiet", "--locked", "--no-default-features",
         "--example", "required_card_fronts", "--", str(source)],
        check=True, capture_output=True, text=True,
        cwd=Path(__file__).resolve().parents[1],
    ).stdout.splitlines()
    required = set(required)
    sources = json.loads((review / "sources.json").read_text())
    cards = [card for sheet in sources.values() for card in sheet]
    faces = {card["identity"]: card for card in cards}
    check_identities("Source mappings", required, set(faces))
    if len(cards) != len(faces):
        raise ValueError("Duplicate identity in source mappings.")
    available = {file.stem: file for file in exports.glob("*.png") if file.is_file()}
    expected = required if mode == "final" else required.intersection(available)
    check_identities("PhotoCraft exports", expected, set(available))
    if not available:
        raise ValueError("Provide at least one PhotoCraft export.")

    manifest = dict(line.split("  ", 1)[::-1]
                    for line in (source / "SHA256SUMS").read_text().splitlines())

    def original(path):
        content = (source / path).read_bytes()
        if sha256(content).hexdigest() != manifest[path]:
            raise ValueError(f"Source digest differs for {path}")
        return content

    catalog_path = "catalog/cards.json"
    catalog = original(catalog_path)
    entries = json.loads(catalog)
    by_id = {entry["card_id"]: entry for entry in entries}
    for identity, card in faces.items():
        entry = by_id[card["card_id"]]
        if any(card[key] != entry[key] for key in ("face", "back", "sheet")):
            raise ValueError(f"Source mapping differs from the audited catalog for {identity}.")

    backs = {card["back"] for card in faces.values()}
    if len(backs) != 1:
        raise ValueError("The required cards must use a single shared back.")
    back = backs.pop()
    files = {back: original(back), catalog_path: catalog}

    checked = {}
    if mode == "final":
        provenance = (review / "provenance.json").read_bytes()
        rows = json.loads(provenance)["faces"]
        checked = {row["identity"]: row for row in rows}
        check_identities("Crop provenance", required, set(checked))
        if len(rows) != len(checked):
            raise ValueError("Duplicate identity in provenance.")
        edited = {}
        for identity, row in checked.items():
            card = faces[identity]
            if row["workshop_id"] != card["card_id"] or row["runtime_path"] != card["face"]:
                raise ValueError(f"Crop provenance differs from source mapping for {identity}.")
            edited[card["card_id"]] = row["bounds_left_top_right_bottom"]
        for entry in entries:
            if entry["card_id"] in edited:
                entry["crop"] = edited[entry["card_id"]]
        files[catalog_path] = (json.dumps(entries, indent=2) + "\n").encode()
        files["provenance.json"] = provenance

    for identity, card in faces.items():
        path = card["face"]
        if path in files:
            raise ValueError(f"Duplicate runtime path for {identity}: {path}")
        if identity in available:
            content = available[identity].read_bytes()
            if mode == "final" and sha256(content).hexdigest() != checked[identity]["export_sha256"]:
                raise ValueError(f"Export digest differs from reviewed crop for {identity}.")
            files[path] = content
        else:
            files[path] = original(path)

    output.mkdir()
    with TemporaryDirectory(dir=output) as temporary:
        staging = Path(temporary)
        sums = []
        for relative, content in sorted(files.items()):
            file = staging / relative
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(content)
            sums.append(f"{sha256(content).hexdigest()}  {relative}\n")
        manifest_bytes = "".join(sums).encode()
        for path in staging.iterdir():
            path.rename(output / path.name)
        (output / "SHA256SUMS").write_bytes(manifest_bytes)
    return sha256(manifest_bytes).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("probe", "final"))
    parser.add_argument("source", type=Path, help="Audited original owner-local collection")
    parser.add_argument("review", type=Path, help="Directory with sources.json and reviewed provenance")
    parser.add_argument("exports", type=Path, help="Directory with PNGs named by printed card ID")
    parser.add_argument("output", type=Path, help="New owner-local collection directory")
    args = parser.parse_args()
    try:
        print(stage(args.mode, args.source.resolve(), args.review, args.exports, args.output))
    except subprocess.CalledProcessError as error:
        print(error.stderr.strip() or str(error), file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError) as error:
        print(error, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
