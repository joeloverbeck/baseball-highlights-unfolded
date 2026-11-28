# Play two consecutive Expert games against a talking manager

Status: specified; the owner has accepted the [eight original-set proxies](../foundation/content.md#starter-card-proxies). The shared [governance extension](https://github.com/joeloverbeck/unfolded-tabletop/pull/114) has merged. The selected Glove and Buy Round design is specified in [#4](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/4). Activate implementation separately.

From a normal Ubuntu launch, select an original team and either Codex or Claude Code. Start game one of a legal Expert series, play a complete mini-game, use its actual revenue to conduct the Buy Round, and play game two with the resulting roster and home-field change. Stop at the recorded second result with the series able to continue. Keep one series and one rules engine throughout. All four original teams and all sixty original free agents are the admission set. Enable only base content for this slice.

## Acceptance through ordinary play

1. Start with shuffled starters, the real six-card market, private six-card hands and optional On Deck. Verify all four fifteen-card rosters, including ST-003 using ST-004 in San Francisco. Inspect the approved fronts and common back, readable pile thickness, speed, threats and occupied bases from the native camera.
2. Exercise legal card selection, target choice, PH, Save and tied extra innings using rules-derived cases. Confirm the RB 10–11 sequence independently and preserve hidden information, including simultaneous commitments.
3. See each revenue contribution and who chooses buy order. Buy, immediately see the refill and decide again. Demote only eligible In Play cards. Verify both 15-card rosters, deck order, score and subsequent draws. Game two must demonstrate the acquisition through actual lineup state and play.
4. Run the same two-game path with each supported provider. Test the game choices from a seat-limited observation. Talk to the opponent about a named play or purchase, get an in-character reply, mute the chat and keep playing. Confirm that typed messages alone cannot apply moves or expose private state.
5. Quit or interrupt during a pending choice and during a buy. On restart, recover same opponent, roster, series, conversation and outstanding decision. Reject stale and duplicate submissions. Cosmetic motion can settle to the already committed board.
6. Supply headless rule, persistence and visibility evidence plus native inspection from normal launch. Record live provider, build revision, seed or replay reference where available, and actual results separately for each path. The owner evaluates the exact combined art, readability, anticipation of consequences and manager conversation.

Refresh the five-repository search in [the engineering foundation](../foundation/engineering.md) before implementation. Use the Cuba Libre pile, Arkham draw/hand, shared shuffle, asset, motion and seat paths as starting evidence. Complete any required extraction/migrations upstream before the dependent Baseball work. Use the three-alternative design workflow for new graphics and layouts. Full Standard/Beginner, later Expert games, expansions and the complete distribution package remain subsequent work. Keep the content ledger current with every delivery.


## Issue #5 implementation

[Sign a Free Agent, reveal the replacement, and demote a player](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/5) exercises the reviewed SF decision after mini-game cleanup. SF has the six reviewed In Play cards, $6 and shops first. NY has six public In Play cards worth $7. Each new series shuffles its hidden Lineups and remaining market tail using a private seed. The save preserves the exact order through restart. In the reviewed state, hands and On Deck are empty under RB 8. Observations expose the viewer’s empty hand and public pile counts. Deck order and the other seat’s hand field stay outside the seat response.

The normal Ubuntu build opens at this decision. The public Buy Row changes as part of the same save transaction as the charge and the face-down Lineup insertion. The manager’s demotion remains pending across restart. The next ticket owns shopping order, additional purchases and round cleanup.

| Repository | Inspected revision | Implementation finding |
| --- | --- | --- |
| unfolded-tabletop | `443ffe135ac3a17cf97c2e1bbc39807ae5fd637c` | Shared `Seat`, `ui_target` and windowless capture drive the same native pointer controls. Consume verified asset reads, recoverable record replacement, seeded shuffle, `card::pile` and `TurningCard`, and `CardMove`. Cargo pins `1d1907d48ecc00fdc6a5e3f673bc5f97da60b53a`, whose package code is identical at the governance revision. |
| Cuba Libre | `ace2696b89e8ba62d373dcf524347b69bdcc7872` | `tabletop::piles::spawn_stack` feeds game-specific thickness, COIN preview and seam spacing to the shared pile mesh. Baseball can supply committed counts directly to that mesh. |
| Arkham | `cf7d92df0433d8c4a2a36621c8e668bee22fb968` | `gathering_tabletop::draws::Playback` sequences private hands, shuffle and recycle. The signing, simultaneous reveal and demotion each use one committed Baseball event and shared card trajectories. |
| Netrunner | `5727b703b2ef7ca5a3886b2874fe50b8d3ec3caa` | `tabletop::seat::Table` identifies native controls from their targets and supplies seat-limited state. Baseball uses that shared adapter contract. `game::turns::setup` keeps shuffle and reveal policy in the game. |
| Mudsong | `9630c8c0bd4567cbaf91f7c14ce81b116f275641` | `seeded_randomness` exercises shared draw and shuffle continuity. Baseball uses the same generator to set up the reproducible position and saves the actual card order. |

The pinned facilities cover the required pile geometry and transfer poses. Issue #5 required no extraction or sibling migration. Baseball owns legal decisions, six stable slots, zones and public disclosure.

The Rust checks exercise the charge, placement, same-slot reveal, second-seat refresh, all six demotion choices, save continuation, private shuffle order and unchanged bytes after a refusal. The windowless Ubuntu Vulkan run activated all six offer inspections and all six demotion inspections through shared pointer controls. It signed Foxx, showed Maris to the already-open NY seat and resumed SF’s pending demotion. The ordinary Ubuntu window rendered the face-down piles on NVIDIA Vulkan, ignored a secondary click on the signing control, accepted a primary click, closed through X11, reopened the same position and demoted ST-004. The owner check covers physical input and readability on the owner’s display.

Consulted PURP-3/5, WORK-1–3/5/6, EVID-3–6, REP-1–3, AUTH-1–3, AGENT-3–5, OPS-1–7, PUB-1, FID-1–4, SIM-4, INT-1/2/4 and LONG-1.

## Issue #6 implementation

[Complete the Buy Round with either manager shopping first](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/6) resumes from the reviewed post-mini-game position. SF's six cards earn $6 and NY's six earn $7. SF chooses the first buyer. A separate tied-revenue fixture gives the choice to NY as the mini-game loser. The two buyers can finish with funds available or sign again after each required demotion. The second pass sends both remaining In Play piles to their face-up Dugouts.

The pinned shared facilities still cover the round. The following normal paths were rechecked for the second buyer, resume and cleanup. The additional decisions, revenue evidence, lineup insertion order and public Dugout projection are Baseball rules. The existing shared pin requires no producer or sibling migration.

| Repository | Inspected revision | Behavior used |
| --- | --- | --- |
| unfolded-tabletop | `443ffe135ac3a17cf97c2e1bbc39807ae5fd637c` | `seat::Game` and `ui_target` drive the same controls for either seat. `save::load` and `save::write` verify records and reject a competing writer under the replacement lock. `card::pile` renders committed counts and `CardMove` presents transfers. These facilities are unchanged from the Cargo pin. |
| Cuba Libre | `ace2696b89e8ba62d373dcf524347b69bdcc7872` | `tabletop::piles::spawn_stack` shows committed thickness and a public top face through shared card primitives. Its COIN reveal rules stay in Cuba Libre. |
| Arkham | `cf7d92df0433d8c4a2a36621c8e668bee22fb968` | `gathering_tabletop::draws::Playback` and the motion adapter map committed draw and recycle events onto shared trajectories. Baseball has the same shared transfer facilities for a single signing and its refill. |
| Netrunner | `5727b703b2ef7ca5a3886b2874fe50b8d3ec3caa` | `tabletop::seat::Table::observe` projects public state and limits each seat to its own choices. Baseball follows the same producer interface and retains its own disclosure rules and decision payload. |
| Mudsong | `9630c8c0bd4567cbaf91f7c14ce81b116f275641` | `seeded_randomness` exercises the shared generator across shuffle and draw. Baseball uses the pinned shuffle to reproduce hidden stacks and validate the committed round. |

The Rust tests exercise both order choices, the loser’s right to choose on a revenue tie, pass with affordable offers, a competing writer, and a reopened second signing from the just-refilled slot. In the separate $11 NY position, the charges are $4 and $7, the slot reveals FA-067 then FA-068, and FA-067 ends above FA-065 on the face-down Lineup. A submitted action names the original card and its slot. A stale revision or mismatched card leaves the replacement in place.

The native Ubuntu smoke exercised SF signing FA-065, resuming the pending demotion, passing with $2, and NY signing FA-067 for $7 to reveal FA-068. Both sides ended at 15 active players with five face-up Dugout cards and zero balance. Card coverage in the ledger applies to these Buy Round operations. Next-game play and physical-input and readability acceptance remain pending.

The following v1 migration records the behavior delivered for issue #6. The [development save policy](../foundation/engineering.md#series-saves-during-development) supersedes its compatibility requirement for subsequent work.

Existing v1 saves retain the same series, physical card identities, Lineup order and committed decisions. An untouched v1 save opens at order selection. A pending demotion remains pending. A completed v1 demotion returns to SF’s shopping turn. FA-068 moves to the next prescribed refill position and the other hidden cards retain their relative order. Opening a v1 save leaves its bytes intact until the next successful choice writes v2. The prior record is retained by shared record replacement. The legacy probe exercised all three entry states with saves produced by the prior implementation.

Issue #6 verified pending-demotion migration with a scratch save from the v1 build. [Historical fixture](https://github.com/joeloverbeck/baseball-highlights-unfolded/blob/af8cb8868982e0ae81ebf43c0ae556d3978cf35d/tests/fixtures/expert-buy-round-v1-pending.json)

Consulted PURP-3/5/6, WORK-1–3/5/6, EVID-3–6, REP-1–3, AUTH-1–3, AGENT-3–5, OPS-1–7, FID-1–4, SIM-4, INT-1/2/4 and LONG-1.

## Issue #7 implementation

[Preview and commit a Glove target](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/7) opens a separate reviewed position at NY 2–SF 1. NY has an average runner on second, Mickey Maris threatens a single followed by a home run, and SF has played ST-001. Each target identifies the public card instance and its printed box number. Preview selection lives in the SF window. Only commitment writes the cancellation, surviving hits and SF's new threats to the save, in that order. The high field camera settles directly to the committed position. The review ends after this play. Drawing Maris from the modified Lineup belongs to the game-two continuation.

The owner-local bundle at `local/issue-7/glove.bundle` contains checkpoint `97109a0f445fb0d4ef66977e37421e9283c3f53b` on `49c7be647431c310e689492167a3630f0ee721ea`. The restored clone passed `git fsck`. The implementation adapted the checkpoint to `a84214d30f0f6542b377b95d4c5b0c5e30f012d0`, including the merged two-manager Buy Round and legacy-save reader.

The native fixture uses the selected Press Box art. The verified export hides the study's sample runner ring and double bar, objects 29 and 30. The field uses simple geometry at the existing marker scale and draws the speed bars from each observed runner. Bespoke 3D pieces are outside the reviewed slice. This VectorCraft 0.24.1 command reproduced the shipped field PNG byte for byte.

```sh
VECTORCRAFT_NO_PREFS=1 vectorcraft-cli --safe-mode run \
  --in art/manager-review/park-0.vectorcraft \
  --cmd object.hide --params '{"ids":[29,30]}' \
  --scale 1 --export /tmp/park-field.png
cmp /tmp/park-field.png art/manager-review/park-field.png
```

| Repository | Inspected revision | Implementation finding |
| --- | --- | --- |
| unfolded-tabletop | `443ffe135ac3a17cf97c2e1bbc39807ae5fd637c` | The existing Cargo pin `1d1907d48ecc00fdc6a5e3f673bc5f97da60b53a` supplies `seat::Game`, `ui_target`, capture, validated record replacement and seeded shuffle. Both Baseball views reuse these facilities. |
| Cuba Libre | `ace2696b89e8ba62d373dcf524347b69bdcc7872` | `tabletop::execution::previewed` and `tabletop::popover::preview_lines` project COIN choices. Baseball supplies its own target and runner consequences to the existing local inspection controls. |
| Arkham | `cf7d92df0433d8c4a2a36621c8e668bee22fb968` | `gathering_tabletop::draws::Playback` and the motion adapter present committed draws using `CardMove`. The Glove field renders the committed runners directly. The existing Buy Round still uses shared card trajectories. |
| Netrunner | `5727b703b2ef7ca5a3886b2874fe50b8d3ec3caa` | `tabletop::seat::Table::observe` limits diagnostics to the viewing seat. Both Baseball windows use the existing shared seat adapter with Baseball's Glove disclosure rules. |
| Mudsong | `9630c8c0bd4567cbaf91f7c14ce81b116f275641` | `seeded_randomness` covers deterministic shuffle and draw. Baseball continues to reproduce physical card instances and hidden order with the pinned generator. |

Issue #7 requires no shared extraction or sibling migration. The [manager-seat tests](../../tests/glove.rs) reproduce both Maris branches, restore the pending manager and hit boxes, reject wrong-seat, stale, duplicate and competing writes, and exercise one target and zero targets. The independent RB 11 example removes Wiffle’s middle box and resolves boxes 1 and 3 in order. Two slow runners occupy first and second before McGwire threatens a single and double. The [native smoke](../../scripts/native-glove) exercises the two reviewed Maris outcomes in the ordinary X11 window. A shared JSON-lines seat run also exercised preview, wrong-seat refusal, commitment, refresh of an already open NY seat and stale refusal. Physical input and readability on the owner's display remain for owner acceptance.

Consulted PURP-3/5/6, WORK-1–3/5/6, EVID-3–6, SEM-1, REP-1–3, AUTH-1–3, AGENT-3–5, OPS-1–7, FID-1–4, INT-1/2/4 and LONG-1.

## Issue #8 implementation

The [game-two continuation](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/8) starts from the completed Buy Round in the same save. SF plays first as Visitor, and both teams draw six from the top of their post-buy Lineups. NY hosts and carries its game-one win. The review covers the first three plays from hand, including any Glove decision they create. On Deck, pinch hitting, the Visitor Save, extra innings and the rest of game two belong to issue #3.

Issue #22 regenerated the scratch starting position at the same SF purchase-order choice in the current exact-replay format. Its SHA-256 is `319eeef67f4b8f98769de7560493e5b293810dd96bd1be11b6156c615811b8d6`. The original issue #8 run used the earlier save with SHA-256 `9a000386da0ca3b9df47ab4caa86ed2276b2d92641a88d3920aa6ebd1836dad4`. [Current scratch save](../../tests/fixtures/expert-game-two-start.json) · [Original issue #8 save](https://github.com/joeloverbeck/baseball-highlights-unfolded/blob/2f4a6cfee1200d57ba07be91d68e00c469c5e06c/tests/fixtures/expert-game-two-start.json)

The [README replay](../../README.md#follow-the-signings-into-game-two) works from a copy. The two signings and demotions preserve both 15-card rosters and each card's instance. In that replay NY's acquired Maris cancels ST-013's single, then threatens a single and a home run. SF's acquired Foxx cancels either box. The home run alone scores one. The single alone leaves the score at 0–0 and puts Maris on first. Foxx then threatens a double. RB 5–6 and Ref 2 supply the timing and two independently calculated projections.

| Repository | Inspected revision | Carryover path |
| --- | --- | --- |
| unfolded-tabletop | `443ffe135ac3a17cf97c2e1bbc39807ae5fd637c` | The existing pin supplies `SeededRandom`, validated record replacement and the shared `seat::Game` pointer controls. The draw consumes the existing ordered pile. |
| Cuba Libre | `ace2696b89e8ba62d373dcf524347b69bdcc7872` | `tabletop::piles::spawn_stack` derives display from committed zones. Baseball uses its own field and hand disclosure on the same shared rendering primitives. |
| Arkham | `cf7d92df0433d8c4a2a36621c8e668bee22fb968` | `gathering_tabletop::draws::Playback` presents committed draws. Baseball's six cards enter the hand together and the existing field camera presents the new position. |
| Netrunner | `5727b703b2ef7ca5a3886b2874fe50b8d3ec3caa` | `tabletop::seat::Table::observe` separates public state from manager-private zones. Baseball's seat projection exposes each hand only to its owner. |
| Mudsong | `9630c8c0bd4567cbaf91f7c14ce81b116f275641` | `seeded_randomness` checks the pinned shuffle-and-draw primitive. Baseball uses the same saved seed for replay validation and keeps its own ordering and reveal rules. |

The current shared pin covers this path. The issue #8 build reopened its Buy Round and standalone Glove saves. Game two uses the approved Press Box camera and the Scorebook’s decision and source-card areas. The hand uses the same one-row player rail as the native Buy Round demotion control. [Selected design](../../art/manager-review/COMBINED.md)

**Manager-seat evidence.** The Rust continuation test compares both saved Lineups with the reviewed order after the purchases and after six cards are drawn per club. It checks each manager's exact private hand and the public Lineup counts through the two seat observations. It also exercises wrong-seat and demoted-card refusals, the public Glove state, restart and duplicate commitment refusal. The Rust recovery test checks the same opening hands after continuing from the restored demotion. [Rust continuation test](../../tests/game_two.rs) · [Rust recovery test](../../tests/series_save.rs)

The issue #8 two-process JSON-lines run used the named controls on both seats, including both local previews, and produced the same committed field in both observations.

**Ubuntu automation.** Two ordinary X11 windows at 1280×800 on NVIDIA GeForce RTX 2080 Ti Vulkan ran the same trace from the same starting save. Both windows showed Maris, Average, on first after the commitment. The 1080×720 capture shows all six hand labels, and the small window accepted inspection and play of Maris. The separate exact-position Glove check still covers NY 2–SF 1 and the Average runner on second.

**Replay identity.** The Ubuntu and JSON-lines runs used the debug executable with SHA-256 `b55667fbb9c9a88df8e9872c88c08ff7413519198a4737d2bc32daec9c5e3f88`. The two resulting saves were byte-for-byte identical with SHA-256 `349832864c46a5deec9bce79c9c2966e123a5327c505215f4e45c2d65af4c43f`. The trace, saves, control labels, captures and build digest are in the owner-local `local/issue-8/replay/` directory.

**Ubuntu owner check.** Pending. Use the README replay and the separate Glove review to locate the runner and read its speed, explain both target outcomes, compare Rizzuto and Foxx at playing size, name Foxx's replacement and SF's remaining funds, and select a demotion. The offer inspector already displays speed, revenue and PH as text. Whether comparison needs those fields directly in the Buy Row depends on the owner's pass at playing size.

Consulted PURP-3/5/6, WORK-1–3/5/6, EVID-3–6/8, SEM-1, REP-1–3, AUTH-1–3, AGENT-3–5, OPS-1–7, PUB-1, FID-1–4, SIM-4, INT-1/2/4 and LONG-1.
