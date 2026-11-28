# Reuse across the Unfolded series

Before drafting or implementing **every spec**, inspect the current shared repository, Arkham Unfolded, Cuba Libre Unfolded, Netrunner Unfolded and Mudsong. Search normal entry points, tests, tooling and native interactions. Record inspected revisions, relevant symbols, what is already shared, what needs extraction and what is specific to baseball in the spec or issue. Refresh this evidence when implementation starts. Respect private sibling custody.

If the library already provides the behavior, consume its immutable revision. If a sibling holds needed common code, extract it into unfolded-tabletop, migrate every affected existing consumer and exercise its real path. Merge the required producer and consumer PRs, pull their main branches, and record merged/pulled revisions and verification before beginning dependent Baseball work. Shared library tests alone cannot demonstrate that the sibling still works. Preserve independent builds, game policy in adapters, and a rollback path. Ordinary builds must not rely on sibling checkouts. Implement genuinely new local behavior only after this search. Shared applicability is established per behavior.

## Series saves during development

Choose the clearest save structure for the rules Baseball implements today. On ordinary reopen, restore the last committed decision boundary. Explicit recovery restores the previous committed boundary into a fresh path. Both must reproduce the selected record's series, card order and pending choice. See the [exact-replay decision](../adr/0001-admit-series-saves-by-exact-replay.md).

Saves from other builds are disposable. Spend no implementation or test effort on compatibility readers, migrations, or retaining obsolete fields. An earlier save may open if it passes the current format and exact-replay checks. If it fails, explain the refusal and preserve the file. The player can explicitly start a series at a fresh path.

The owner will first review this policy when the original competitive two-team rules for Beginner, Standard and Expert and all 60 original starter and 60 original free-agent cards are implemented. The policy continues until the owner changes it. At that review the owner may extend the disposable period through all published expansion cards, including cards acquired later.

The [Buy Round v1 migration](../specs/two-expert-games.md) records an earlier delivery. Its pending-demotion test required new code to reconstruct an obsolete round and reorder a hidden refill. That historical promise is superseded by this policy. Keep the pending demotion reproducible in the current build.

## First-spec source trace, 2026-11-27

| Repository and inspected local revision | Implementation evidence and disposition |
|---|---|
| unfolded-tabletop · ea5a9b19 | src/card.rs owns card solids; src/card_motion.rs owns card trajectories. random/ supplies seeded draws of indices and shuffle. assets/, record-replacement/, seat-protocol/, seat-client/, agent-players/ and src/seat/ offer asset admission, safe persistence and agent communication building blocks. Inspect feature flags and current game pins before integration. |
| cuba-libre-unfolded · ace2696b | src/tabletop/piles.rs layers the shared rounded card solid, showing a seam every six or seven cards. It uses shared motion for lift, cross and flip; committed model counts and rest poses survive interruption. Inspect layers and advance paths for a reusable stack display. Its face-up COIN preview is a game-specific disclosure rule. src/faction_seats.rs, seat_wait.rs and save.rs are precedents for waiting, resumption and validated moves. |
| arkham-unfolded · cf7d92df | src/gathering_tabletop/motion.rs tracks physical IDs and maps observed source and destination into shared CardMove; draws and shuffle presentation live alongside it. Follow the actual draw path when designing six revealed free agents and a one-slot refill. Hand layout and investigator visibility belong to Arkham. src/partners/ and src/gathering_tabletop/talk/ are precedents for a returning seat and two-way conversation. |
| netrunner-unfolded · 5727b703 | src/game/turns/setup.rs and src/tabletop/seat.rs provide comparison paths for hidden decks, owner-relative observations and human decisions. src/assets.rs covers artwork admission; inspect the real textured-card and token paths before designing replacements. |
| mudsong · 9630c8c0 | crates/rules/src/play.rs and crates/rules/tests/seeded_randomness.rs exercise seeded randomness. Inspect its local implementations and tests during the spec search. Keep its private game material local. |

A free-agent purchase decides which visible card moves to whose Lineup, what is demoted and when the new row member becomes available. Those are Baseball rules. The mechanics of a face moving from a known deck to an observed public slot, deck shuffling, a pile showing its count, and seat reconnection are concrete reuse candidates. Trace each independently before writing code.

## EVID-6 extension accepted 2026-11-27

The previous Cuba Libre exception ends at a fifth game. The owner explicitly accepted a Baseball extension to the normal three-case and independent-holdout requirement for concrete common mechanisms in Baseball plus at least one existing Unfolded game. Two actual consumers with different game adapters provide the evidence for this limited use. Two cases can share an unrecognized assumption. Verify each affected app, preserve rollback, merge and pull migrations, and reassess after two active hours per extraction. This confers no claim of general framework fitness. Reassess for an additional game, general publication, or loss of the second real consumer. The shared repository merged the extension in PR #114 and pulled it to main at `443ffe135ac3a17cf97c2e1bbc39807ae5fd637c`. Each extraction and its consumer migrations still require verification, merge and pull before dependent Baseball work. [Shared governance decision](https://github.com/joeloverbeck/unfolded-tabletop/pull/114)

## Constitution and skills

Core and modules come from principles-for-repos tag 2026-11-23.1, commit 263d9d28d6164d7d82cd59d99db014889cc059a4. Fidelity applies to published rules, agency-simulation to runtime managers, integration to cross-repo migration and private assets, longitudinal to a persisted series with a returning participant. Revisit other triggers when a spec makes new claims. Matt Pocock’s engineering and productivity directories were installed directly from upstream commit bbadaa631c37af594e5a05c9cefacd9956136b18. Repository workflow lives in docs/agents. Preserve upstream attribution when refreshing.
