# Card and component inventory

Owner-local root on Ubuntu: `~/Pictures/Baseball Highlights: 2045 Resources/`. The Workshop save is 1803946195. Its catalog maps TTS IDs to sheets, crop rectangles, PNG fronts, instances and a common JPEG back. The original SHA256SUMS verifies the extraction. Store supplemental PDFs in its rules directory. Use a configurable resource root and keep original files intact. A later import should accept a single ZIP, validate required files, versions and hashes, unpack to staging, and activate the complete package atomically. Report missing identities before admitting play. Publishing an asset pack in GitHub Releases requires established distribution rights.

## Original set admission

On 2026-11-27 inspection of the printed ID fields, deck instances and relevant sheet cells found the following. Each original team deck has 15 instances. Each has only 13 distinct printed identities. Numerically adjacent IDs represent separate physical cards even when their game statistics agree.

| Team | Missing original face numbers | Repeated numbers in its deck |
|---|---|---|
| San Francisco | ST-003, ST-009 | ST-008, ST-010 |
| Los Angeles | ST-019, ST-024 | ST-018, ST-025 |
| Boston | ST-034, ST-039 | ST-033, ST-040 |
| New York | ST-048, ST-055 | ST-049, ST-054 |

The 60-card free-agent deck contains FA-061 through FA-120, once each. All 536 catalogued faces exist and use the same present back. The two missing URLs in the extraction report refer to a custom surface and its placeholder. Counts and hashes alone cannot detect a deck assembled with duplicate artwork. Preserve the original extraction. Construct each logical team from its fifteen consecutive starter identities, one of each. Use the following eight exceptions when resolving a logical identity to card data and an English face.

### Starter-card proxies

The owner approved the following existing faces and gameplay equivalents. In the ledger, `proxy:ST-nnn` identifies the donor of the printed stats and displayed face. The blank Workshop ID and instance count on the recipient mean its exact scan is absent. Keep its own ST identity in the roster, save, logs and inspection UI. If exact English art becomes available, verify it and replace the proxy for that identity. Gameplay implementation and verification are still pending for all eight.

| Team | Logical identity → approved donor |
|---|---|
| San Francisco | ST-003 → ST-004, ST-009 → ST-010 |
| Los Angeles | ST-019 → ST-018, ST-024 → ST-025 |
| Boston | ST-034 → ST-033, ST-039 → ST-040 |
| New York | ST-048 → ST-049, ST-055 → ST-054 |

San Francisco needs a composition correction. Keep one ST-008. Use ST-004’s data and face for the missing ST-003 slot, and keep ST-004 itself. Importing the Workshop deck as dealt would leave an extra ST-008 in that slot. For each team, check all fifteen distinct logical IDs and their resolved donors; check all sixty free agents once each. When the importer lands, exercise these same identities and approved donors on its actual roster and Free Agent deck output, including all four team boundaries.

Source: gkrands reported comparing the Russian TTS version on 12 March 2022. The mapping is an attributed community comparison. The local audit confirms the donor assets, deck composition and common back. Independent inspection of the eight missing originals remains open as an artwork follow-up in issue #2. [Workshop discussion](https://steamcommunity.com/sharedfiles/filedetails/?id=1803946195)

## Remaining content

[Card content](card-content.csv) tracks every local Workshop card ID, including alternative starter teams, parks and travel stadium cards. Multiple source images or repeated game instances may refer to the same printed identity. Named park records accommodate landscape faces. The final columns track implementation, verification and evidence independently from artwork availability. Filter on pending/blocked to see outstanding work. Update affected records in the implementing PR with the rules reference, evidence and issue/commit. Verify playable identities through real deals/choices, applicable rules, artwork and persisted state. Keep duplicate sources traceable while counting a single rules implementation per equivalent behavior.

The ledger also lists the absent original cards, the unlocated AS-591–605 range and unmatched promo numbers listed by the reference booklet. Its promo ranges, component count and actual promo types need edition reconciliation; never infer a missing player solely from a gap in numbering. Reconcile additional published starter variants and promos against the publisher inventory after owned coverage. Booklet Ref 7 calls for 15 Coaches and prints an inclusive range ending in 136; local Coach faces run CO-121–135. Record contradictions where discovered. Keep later gaps visible without blocking the owned collection.

## Physical components and digital obligations

RB 2 lists twelve 15-card starting decks in Bases Loaded, sixty original free agents, eleven expansion decks, eighteen promo cards, twenty-five dividers, four stadium mats, sixty pawns in three colors, and four each of runs scored, games won and Home/Visitor markers. It includes rules, reference and guided-play books. The original edition instead supplied four starter teams. Dividers become set/roster organization. Mats define zones, bases, In Play, Lineup, Dugout, Minors and On Deck. The market has its own deck and row. Provide separate bases and threats for each side even within a single scenic ballpark. Score and pennant markers can become a scoreboard and host indicator. Batters at home and runners on base retain their side, speed and associated threat. Preserve score, occupied bases, ordered threats and hidden-card backs at a glance.
