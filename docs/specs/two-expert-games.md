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

Existing v1 saves retain the same series, physical card identities, Lineup order and committed decisions. An untouched v1 save opens at order selection. A pending demotion remains pending. A completed v1 demotion returns to SF’s shopping turn. FA-068 moves to the next prescribed refill position and the other hidden cards retain their relative order. Opening a v1 save leaves its bytes intact until the next successful choice writes v2. The prior record is retained by shared record replacement. The legacy probe exercised all three entry states with saves produced by the prior implementation.

The Rust suite now exercises the pending-demotion migration with a [scratch save generated by the v1 build](../../tests/fixtures/expert-buy-round-v1-pending.json).

Consulted PURP-3/5/6, WORK-1–3/5/6, EVID-3–6, REP-1–3, AUTH-1–3, AGENT-3–5, OPS-1–7, FID-1–4, SIM-4, INT-1/2/4 and LONG-1.

## Issue #7 local handoff

The unfinished [Glove target work](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/7) is preserved in the owner-local `local/issue-7/glove.bundle`, under the checkout's ignored `local/` directory. Branch `issue-7-checkpoint` contains commit `97109a0f445fb0d4ef66977e37421e9283c3f53b` on base `49c7be647431c310e689492167a3630f0ee721ea`. The bundle includes the eight modified files and four formerly untracked files. A fresh clone of the bundle passed `git fsck` and matched all twelve draft files byte for byte.

On activation, restore the checkpoint from the repository root and compare it with current `main` before transferring the Glove changes. The draft predates the merged Buy Round and desktop-seat changes.

```sh
git clone --branch issue-7-checkpoint local/issue-7/glove.bundle local/issue-7/restore
```
