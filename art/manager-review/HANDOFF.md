# Two manager decisions · design review

Prepared for a narrower design/interaction spec under Baseball issue #3. The parent owns two complete competitive Expert games, agent seats, chat and save recovery. This round supplies design alternatives and reproducible decision examples. Owner selection belongs in the subsequent spec. The browser review is a fixture that advances between recorded states. Native play, a complete rules engine and live providers require separate evidence.

## Review and restore

On the owner machine open art/manager-review/review.html in the Baseball checkout. Press Box prioritizes the field with a persistent decision panel; Scorebook puts card evidence and projected consequences into columns; Dugout Rail gives the six-card market the most space and shows targets below a low field view. The proposed default for selection is Press Box. The comparison uses two independently selectable snapshots and supports a guided purchase, choice of demotion, game-two transition and both targets. The roster arithmetic and the example New York reply are fixed.

The local six-frame manager-decisions.figcraft includes private card images. This branch carries the original park designs, FigCraft layer data and a local restoration recipe. With Pillow installed, run from this branch:

```sh
python3 art/manager-review/restore.py "$HOME/Pictures/Baseball Highlights: 2045 Resources"
```

Open the restored file in FigCraft. Export top-level frame IDs 2/24 (Press Box), 61/85 (Scorebook) and 131/150 (Dugout Rail) to 0-glove.png/0-buy.png, 1-glove.png/1-buy.png and 2-glove.png/2-buy.png respectively, in the same directory as review.html. In each pair the first frame is the Glove scene. Rendered frames stay private. Each ballpark is also editable in the corresponding park-N.vectorcraft file.

## Fixture and expected observations

Game 1 Buy Round. SF has ST-002, 004, 005, 008, 011 and 014 In Play. Printed revenue is 2+2+1+1+0+0 = 6. NY has 7. SF chooses first purchase. The market contains FA-061 to FA-066. Foxx costs 4 and immediately reveals FA-067 Mickey Maris to both managers. SF has 2 remaining, below the lowest available price. Allow any of those six eligible demotions. Foxx goes atop the SF Lineup. In this fixture NY signs Maris for 7, refills with FA-068 and demotes one eligible player. Both carry 15 active players into game two. Verify actual next draws when implementing. The review simulates only this purchase sequence.

Game 2 is in New York. NY leads 2–1, with an average runner on second; Maris has threatened single then home run. SF plays ST-001, a Glove and a single. Cancelling the single produces two NY runs and empty bases (4–1); cancelling the homer scores the average runner plus places the batter on first (3–1). Then SF threatens its printed single. Card data comes from the real collection. These are constructed positions using RB 3, 5–6, 9 and Ref 1–3. Test a full legal path separately. Graphics are 2D camera studies for the native 3D park.

## Proposed testing seam

Drive a single series through the same seat-relative observe/submit boundary used by the native manager and agent adapter. Compare the next observation, legal choices, public events, balance, roster and save/resume continuation. Check both seats for hidden information, including facedown cards, while the replenished six-slot row is public. Refuse stale purchases without adding a second player or repeating a refill. Use the ordinary Ubuntu launch for the separate native readability and input check. The prototype browser checks exercised six loaded frames, six demotion choices displayed, one demotion per direction, and both target outcomes per direction. They do not establish native behavior.

## Source refresh for the two scenes

| Checkout (local HEAD) | Concrete precedent | Disposition and integration evidence needed |
|---|---|---|
| unfolded-tabletop ea5a9b19 | card solid and pile geometry, card_motion::CardMove, SeededRandom::shuffle, shared seat and transport | Consume immutable revision. Preserve the seed and actual deck order. Shared trajectory math can serve signing, revealing and drawing. |
| Cuba Libre ace2696b | tabletop/piles.rs layers, height and committed face/count presentation. Named tests cover one seam per six to seven cards, stack height, model counts and the in-flight turn. | Extract common stack layering if the selected scene uses it. Verify Cuba Libre’s existing preview, landing and restart through its real tray. Keep COIN preview policy in its adapter. |
| Arkham cf7d92df | gathering_tabletop/draws.rs Playback::new, take_transfers, advance; motion.rs maps visible IDs to CardMove. Talk tests exercise actual socket exchanges. | For the six-slot public refill, reuse shared motion and trace which scheduling or transfer bookkeeping is common. Keep investigator hands, weaknesses and deck visibility in Arkham. Verify the original draw/reveal route when migrating. |
| Netrunner 5727b703 | tabletop/seat.rs uses shared start, capture and guarded seat; strips a hidden-hand diagnostic. game/turns/setup.rs implements opening draws and mulligan. | Reuse the shared seat pattern; verify both views when purchasing from a public market into a private lineup. Baseball chooses when information becomes public. |
| Mudsong 9630c8c0 | rules seeded-randomness tests check same-seed reproducibility and historical sequence. | Consume the existing shuffle; preserve consumer-specific shuffle triggers. No migration indicated for that path. |

Cuba Libre’s unrelated local art studies were left in place. Source review establishes candidate ownership. Before dependent implementation, refresh exact revisions, migrate affected siblings, run their real checks, merge required PRs and pull main. The owner-accepted governance extension is proposed in [unfolded-tabletop #114](https://github.com/joeloverbeck/unfolded-tabletop/pull/114). This design round opens no extraction.

## To-spec handoff

After owner review, record the chosen direction or hybrid, revisions and agreed testing seam. Invoke to-spec for this interaction slice, linked under [the existing first playable](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/3). The first playable continues to own both complete games and selectable providers. The design slice defines board information, player actions, visible consequences and recovery; actual ticket activation and native acceptance follow independently. Keep the image bundle owner-local. Consulted PURP-3, WORK-1–3/5/6, EVID-3–6, REP-1–3, AUTH-1–3, OPS-1/4/6/7, AGENT-1/2/5, GOV-1, PUB-1 and the adopted fidelity and integration boundaries.
