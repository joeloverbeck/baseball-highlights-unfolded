# Card and component inventory

Owner-local root on Ubuntu: `~/Pictures/Baseball Highlights: 2045 Resources/`. The Workshop save is 1803946195. Its catalog maps TTS IDs to sheets, crop rectangles, PNG fronts, instances and a common JPEG back. The original SHA256SUMS verifies the extraction. Store supplemental PDFs in its rules directory. Use a configurable resource root and keep original files intact. A later import should accept a single ZIP, validate required files, versions and hashes, unpack to staging, and activate the complete package atomically. Report missing identities before admitting play. Publishing an asset pack in GitHub Releases requires established distribution rights.

## Extract card artwork in PhotoCraft

The desktop and agent seats load the [recovered artwork collection](#recovered-card-artwork) by default from `~/Pictures/Baseball Highlights: 2045 Recovered/`. Its base is the approved 35 PhotoCraft fronts and the original shared back. The base PhotoCraft pack and the original Workshop extraction can also be opened with `--assets`.

Use the project's PhotoCraft MCP to crop and export each card extracted from a sheet. The Workshop catalog locates the source sheet and cell. Measure the actual card bounds in the source pixels because the nominal grid can cut into a card or include its neighbor. Inspect all four edges and corners, confirm the printed identity and orientation against the ledger, and preserve the illustration, text and game symbols. Inspect the full bounds of standalone fronts or backs in PhotoCraft and keep the original file when its bounds are correct. Flag source damage that would require clipping the print or reconstructing it.

PhotoCraft reads from the checkout and exports under `art/`. Copy verified sources to a Git-ignored working directory under `art/` and export the crops there. Record the PhotoCraft version, source digest, source card and printing identity, reviewed pixel rectangle and export digest in the owner-local collection. Keep source files, crops and proofs private. Stage the exported PNGs with their catalog and SHA-256 manifest, then inspect the staged collection through the ordinary desktop launch with `--assets` and a scratch save before changing the default. The runtime consumes verified image files.

In the first Ubuntu handoff for private visual review, give the absolute folder, exact filenames and intended review size. Include a copyable command such as `xdg-open '/absolute/path/to/review-folder'` alongside the preview links.

For a working directory at `art/.card-crops/`, open a sheet with `doc_open(path="art/.card-crops/sheet.jpg")`. After running `image.crop` with the measured pixel bounds, export with `doc_save(path=".card-crops/card.png")`. The output lands at `art/.card-crops/card.png`.

Stage with `python3 scripts/stage_card_crops.py final SOURCE REVIEW EXPORTS OUTPUT`. The helper compares the exported printed IDs against the required faces loaded by the Expert and Glove paths. `REVIEW` contains the private source mappings and crop provenance. Use `probe` for a partial package that fills the other required faces from the original collection.

## Original set admission

On 2026-11-27 inspection of the printed ID fields, deck instances and relevant sheet cells found the following. Each original team deck has 15 instances. Each has only 13 distinct printed identities. Numerically adjacent IDs represent separate physical cards even when their game statistics agree.

| Team | Missing original face numbers | Repeated numbers in its deck |
|---|---|---|
| San Francisco | ST-003, ST-009 | ST-008, ST-010 |
| Los Angeles | ST-019, ST-024 | ST-018, ST-025 |
| Boston | ST-034, ST-039 | ST-033, ST-040 |
| New York | ST-048, ST-055 | ST-049, ST-054 |

The 60-card free-agent deck contains FA-061 through FA-120, once each. All 536 catalogued faces exist and use the same present back. The two missing URLs in the extraction report refer to a custom surface and its placeholder. Counts and hashes alone cannot detect a deck assembled with duplicate artwork. Preserve the original extraction. Construct each logical team from its fifteen consecutive starter identities, one of each. Use the following eight equivalents for gameplay data and for artwork whenever the collection uses the Workshop donor.

### Starter-card proxies

The owner approved the following gameplay equivalents. In the ledger, `proxy:ST-nnn` identifies the artwork donor when the exact face is missing or the donor is clearer at desktop size. Blank Workshop IDs and instance counts on these eight records indicate gaps in the original extraction. Preserve each logical identity in the roster, save and technical records. Across the app, identify players by card name in labels, actions, history and inspection. Keep collector numbers printed on the original artwork intact. Verify independently sourced faces against their identity and gameplay fields and review them alongside the donor before adopting them. The ledger records gameplay implementation and verification for each card.

| Team | Logical identity → approved donor |
|---|---|
| San Francisco | ST-003 → ST-004, ST-009 → ST-010 |
| Los Angeles | ST-019 → ST-018, ST-024 → ST-025 |
| Boston | ST-034 → ST-033, ST-039 → ST-040 |
| New York | ST-048 → ST-049, ST-055 → ST-054 |

San Francisco needs a composition correction. Keep one ST-008. Use ST-004’s data and face for the missing ST-003 slot, and keep ST-004 itself. Importing the Workshop deck as dealt would leave an extra ST-008 in that slot. For each team, check all fifteen distinct logical IDs and their resolved donors; check all sixty free agents once each. When the importer lands, exercise these same identities and approved donors on its actual roster and Free Agent deck output, including all four team boundaries.

On 12 March 2022 gkrands reported comparing the Russian TTS version. The local audit confirms the donor assets, deck composition and common back. [Workshop discussion](https://steamcommunity.com/sharedfiles/filedetails/?id=1803946195)

### Recovered card artwork

The November 28 research pack matches 38 ledger identities. Its source archive, checklist, crops and PhotoCraft review are owner-local. The reviewed collection at `local/card-crop-review/face-pack-2026-11-28/assets-corrected` was installed as the default in `~/Pictures/Baseball Highlights: 2045 Recovered/` with its checksum manifest and audited `catalog/recovered-starters.json` mapping. The current Expert path can display ST-055 in New York's game-two hand. The corrected Los Angeles and Boston fronts are available to the loader and await gameplay for those teams. The Workshop and earlier PhotoCraft collections use donor art. The eight approved gameplay equivalents apply in all collections.

| Identity | Artwork in the recovered collection or research |
| --- | --- |
| ST-019 | Rectified LA paper card from BGG image 2845196. PhotoCraft curves bring its tones closer to ST-018 and preserve a clear face at inspection size. |
| ST-024 | Rectified LA paper card from BGG image 2591141. PhotoCraft curves and sharpening improve readability. The artwork still has softer fine detail than ST-025. |
| ST-034, ST-039 | Rectified Boston paper cards from BGG images 2907228 and 2907226. PhotoCraft curves and detail filters improve readability. Grain remains visible at inspection size. |
| ST-048 | Retain ST-049. PhotoCraft reduces the warm cast, and the photo retains visible curvature and uneven edges. |
| ST-055 | Crop of the Buy Round in Peter Kossits’s licensed iPad adaptation. The visible footer reads ST-055. The numbered paper face remains unlocated. |
| ST-003, ST-009 | Retain ST-004 and ST-010. Exact English faces remain unlocated. |
| AS-591–605, PR-518–520, PR-606–614 | English faces available in the private collection. PR-518 uses the Minnesota version reproduced in the publisher’s update. |
| PR-513, PR-514 | Foil printing evidence for PR-511 and PR-512 respectively. The photographed footers show those base numbers. The reflective, incomplete photographs serve as printing evidence. |
| PR-515 | Shared reference for Visitor travel stadium (Workshop 2704) and Home travel stadium (2705). Each existing source keeps its four Workshop instances. |

The ledger’s `present:photograph`, `present:licensed-app` and `present:private` entries record artwork in the private collection. `evidence:foil` records a photographed printing, and `present:components` points to the existing component records. The CSV and per-face provenance in the private collection record the source image, crop and digest. Issue #2 tracks coverage. [ST-019 source photo](https://boardgamegeek.com/image/2845196/baseball-highlights-2045) · [ST-055 iPad screenshot gallery](https://appshunter.io/ios/app/baseball-highlights-2045/id1003138996)

#### Restore the default collection

Run this from the checkout. If the staged pack is on another machine, point `baseball_pack_source` at a complete owner-local backup of the approved recovered collection. The command verifies the manifest and every listed file before moving the staged copy into place. It requires an unused destination.

```sh
(
    set -eu
    baseball_pack_source="$PWD/local/card-crop-review/face-pack-2026-11-28/assets-corrected"
    baseball_pack_destination="$HOME/Pictures/Baseball Highlights: 2045 Recovered"
    if [ -e "$baseball_pack_destination" ] || [ -L "$baseball_pack_destination" ]; then
        printf 'The destination already exists at %s.\n' "$baseball_pack_destination" >&2
        exit 1
    fi
    mkdir -p -- "$HOME/Pictures"
    baseball_pack_stage=$(mktemp -d -- "$HOME/Pictures/.baseball-recovered.XXXXXX")
    cp -R -- "$baseball_pack_source/." "$baseball_pack_stage/"
    (
        cd -- "$baseball_pack_stage"
        printf '%s  SHA256SUMS\n' 'd858a56885c312cd45c22c45868d81692acacfe5f92d2ba7a591a801d68d40a3' |
            sha256sum --check --quiet
        sha256sum --check --quiet SHA256SUMS
    )
    mv -T --no-clobber -- "$baseball_pack_stage" "$baseball_pack_destination"
    test ! -e "$baseball_pack_stage"
)
```

To launch with the earlier PhotoCraft collection and its donor artwork, run this from the checkout.

```sh
cargo run --locked -- --assets "$HOME/Pictures/Baseball Highlights: 2045 PhotoCraft"
```

## Remaining content

[Card content](card-content.csv) tracks every local Workshop card ID, including alternative starter teams, parks and travel stadium cards. Multiple source images or repeated game instances may refer to the same printed identity. Named park records accommodate landscape faces. The final columns track implementation, verification and evidence independently from artwork availability. Filter on pending/blocked to see outstanding work. Update affected records in the implementing PR with the rules reference, evidence and issue/commit. Verify playable identities through real deals/choices, applicable rules, artwork and persisted state. Keep duplicate sources traceable while counting a single rules implementation per equivalent behavior.

The ledger also lists original starter gaps, the researched All-Star and promo cards, and unmatched promo numbers from the reference booklet. Its promo ranges, component count and actual promo types need edition reconciliation; never infer a missing player solely from a gap in numbering. Reconcile additional published starter variants and promos against the publisher inventory after owned coverage. Booklet Ref 7 calls for 15 Coaches and prints an inclusive range ending in 136; local Coach faces run CO-121–135. Record contradictions where discovered. Keep later gaps visible without blocking the owned collection.

## Physical components and digital obligations

RB 2 lists twelve 15-card starting decks in Bases Loaded, sixty original free agents, eleven expansion decks, eighteen promo cards, twenty-five dividers, four stadium mats, sixty pawns in three colors, and four each of runs scored, games won and Home/Visitor markers. It includes rules, reference and guided-play books. The original edition instead supplied four starter teams. Dividers become set/roster organization. Mats define zones, bases, In Play, Lineup, Dugout, Minors and On Deck. The market has its own deck and row. Provide separate bases and threats for each side even within a single scenic ballpark. Score and pennant markers can become a scoreboard and host indicator. Batters at home and runners on base retain their side, speed and associated threat. Preserve score, occupied bases, ordered threats and hidden-card backs at a glance.
