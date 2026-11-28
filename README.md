# Baseball Highlights: 2045 Unfolded

An Ubuntu-first native Rust tabletop game in development. Manage a team through a competitive series against Codex or Claude Code. Make the lineup, tactical and buying decisions, talk baseball with the opposing manager, and watch the consequences unfold in a 3D ballpark.

The native review opens the Expert Buy Round and follows its purchases into the opening of game two. See the [first playable specification](docs/specs/two-expert-games.md) for the full series work. The [original-card audit](docs/foundation/content.md#starter-card-proxies) records approved equivalents for eight starter faces. All four original teams can be represented with the available artwork.

## Open the reviewed Buy Round

On Ubuntu with Rust 1.100 and a Vulkan adapter, run `cargo run --locked` from the checkout. The game reads the private collection from `~/Pictures/Baseball Highlights: 2045 Recovered`, including the approved PhotoCraft fronts, shared back and the licensed iPad face for ST-055. The four corrected Los Angeles and Boston faces are in the collection for when those teams become playable. See [restore the default collection](docs/foundation/content.md#restore-the-default-collection) to install the recovered pack or launch with the earlier PhotoCraft pack. The original extraction remains available at `~/Pictures/Baseball Highlights: 2045 Resources` via `-- --assets DIRECTORY`. The game reports any missing or changed required face before opening the table.

SF earns $6 and NY earns $7. In the SF manager seat, choose **SF buys first** or **NY buys first**. To replay the reviewed round, choose SF. Hover over **Frankie Foxx** or move keyboard focus to his face. The help beside the face shows his printed symbols, $4 price and $2 remaining after signing. Select **Sign** below Foxx to commit. Mickey Maris takes the same slot. Inspect an eligible In Play player and select **Send to Minors** to return SF to 15 active players. Select **Finish buying** to pass the turn and expire the $2.

Open the same save with `--seat ny`, inspect **Mickey Maris**, then select **Sign** to spend $7. Babe Bench takes the slot. Choose NY’s demotion and select **Finish buying**. On **Buy Round complete**, open either team’s Dugout to inspect its five public faces. **Round history** shows both managers’ choices. Press Space to finish card motion. Use Tab and Shift+Tab to move between faces, help and actions, Enter to activate a focused action, and Escape to dismiss card help.

The default save is `baseball-highlights-unfolded/expert-buy-round.json` under `$XDG_STATE_HOME` or `~/.local/state`. Use `-- --save FILE` with a fresh path to start at the purchase-order choice. Two desktop windows using the same save can follow each other’s turns. Close and relaunch to resume whichever choice is pending. For an isolated agent seat, add `--agent-seat sf` or `--agent-seat ny` after `--`. The seat serves the shared JSON-lines protocol over standard input and output.

This development build reopens saves it can reproduce with its current rules and save format. A refused save stays in place. To start a fresh series, use `--save FILE` with a new path.

## Recover the previous decision

If the active save is missing or damaged, open its previous committed decision in a fresh save.

```sh
cargo run --locked -- --recover-from /path/to/original.json --save /path/to/recovered.json
```

The game checks `original.json.previous` against the current rules and review mode before creating `recovered.json`. Use a destination with neither an active nor a previous record. Add `--review-glove` for a standalone Glove save. The original files stay available to you. For later launches and the other seat, open `--save /path/to/recovered.json` directly.

Each new decision rotates the active save into its `.previous` record. If the existing `.previous` record is unusable, the game reports the failed write and keeps the last committed position and both files.

## Follow the signings into game two

Use the recorded starting position to repeat the same deals in two Ubuntu windows. Run this block in the SF terminal.

```sh
baseball_review_dir=$(mktemp -d)
cp tests/fixtures/expert-game-two-start.json "$baseball_review_dir/series.json"
printf '%s\n' "$baseball_review_dir"
cargo run --locked -- --save "$baseball_review_dir/series.json" --seat sf
```

In a second terminal, replace `/path/to/scratch` with the printed directory and run the NY window.

```sh
cargo run --locked -- --save /path/to/scratch/series.json --seat ny
```

In SF choose **SF buys first**. Inspect **Peewee Rizzuto** in slot 2 and **Frankie Foxx** in slot 5. Both cost $4. Sign Foxx and check that Mickey Maris fills slot 5 and SF has $2 left. Inspect **Rookie Natural** in the second In Play slot from the left. Select **Send to Minors**, then **Finish buying**. In NY sign **Mickey Maris**, check that **Babe Bench** fills the slot, demote **Veteran Natural** in the second In Play slot, and finish buying. The completed round shows **10 in Lineup + 5 in Dugout = 15 active** for both clubs.

In SF open both public Dugouts, then select the pennant **Start Game 2** control. NY hosts and leads the series 1–0. Inspect **Veteran Natural** in the top middle of SF’s hand and select **Play** below its face. In NY inspect **Mickey Maris** at the top left and select **Play**. Maris cancels SF’s single and threatens a single and a home run. In SF inspect **Frankie Foxx** at the top left and select **Play** again.

Compare both Glove outcomes beside the live field. **Cancel single** previews one run for NY and empty bases. **Cancel home run** previews a 0–0 game with Maris on first. Select **Cancel home run**, then **Use Glove / Cancel home run**. Both windows show Maris on first with Foxx’s double waiting at home.

## Review the Glove decision

Run `cargo run --locked -- --review-glove` to open the high field camera as SF. NY leads 2–1 with an Average runner on second. Select either outcome to preview it while the live field holds. Select **Cancel single**, then **Use Glove / Cancel single**, for NY 4–SF 1 with empty bases. Select **Cancel home run**, then **Use Glove / Cancel home run**, for NY 3–SF 1 with Mickey Maris on first. In either case SF’s single then waits at home, and the scorecard lists the committed events in order.

Close and reopen the same save to resume. Add `--seat ny` in a second window to follow the public field as NY. The review has its own `glove-review.json` in the same state directory as the Buy Round. Use `--review-glove --save FILE` with a fresh path for the other outcome. The review ends after this play.

## Repository checks

CI runs `cargo fmt --check`, the Buy Round rules and save tests with `cargo test --locked --no-default-features`, and `cargo check --locked --all-targets` for the Ubuntu app. With Python 3.10+ and Pillow installed, run `python3 -B -m unittest discover -s tests -v` for the original-set ledger, design restore, and game-two fixture checksum.

Run `scripts/native-buy-round` on Ubuntu with Xvfb, xauth, xdotool, Pillow with X11 capture support, and the audited owner-local card collection. It uses the normal desktop binary on an isolated display to exercise primary and secondary clicks, both signings and refills, close and resume, and the face-down piles and Dugouts. It prints the adapter and the temporary directory containing captures, logs, and the scratch save. Use `--assets DIRECTORY` for another copy of the collection or set `WGPU_ADAPTER_NAME` to select an available Vulkan adapter.

Run `scripts/native-buy-round --order` to inspect the exact revenue faces from both manager seats and choose either first buyer. The check uses an isolated X11 display for the 1080 × 720 and 3840 × 2160 client sizes, then records the screenshots and seat transcript in the printed evidence directory.

Run `scripts/native-buy-round --market` for six stable offers, beside-card help, pointer and keyboard input, and an unaffordable offer. Run `scripts/native-buy-round --signing` for signings, transfers, both demotions and recovery. Both routes exercise the manager seats and native X11 clients at 1080 × 720 and 3840 × 2160.

Run `scripts/native-buy-round --game-two` for the full Foxx and Maris replay from buy order through Glove. Both JSON-lines seats and normal X11 clients at 1080 × 720 and 3840 × 2160 exercise the route. The test checks both Glove previews, the save digest after each replayed click and the final saves byte for byte. The printed evidence directory contains the control trace, captures, scratch saves and SHA-256 digests for the executable and starting position. Open a scratch copy of its `sf-hand-position.json` for additional card inspections.

Run `scripts/native-buy-round --start-game` to inspect both public Dugouts from both seats and start game two. It verifies the modified opening hands and exercises the ordinary X11 window at 1080 × 720 and 3840 × 2160.

Run `scripts/native-stadium` to replay the Visitor opener through both manager seats and the ordinary X11 app at 1080 × 720 and 3840 × 2160. The route checks the exact physical card, private hands, keyboard inspection, slow pointer travel across all six card gaps and the anchored stadium. It saves captures, the measured geometry, the manager transcript and the executable digest in the printed evidence directory.

Run `scripts/native-glove` with the same prerequisites to exercise both Glove commitments from the game-two and practice positions at both client sizes. It checks preview switching, the NY observer and reopening and recovering the pending SF choice.

If Xvfb reports `failed to bind listener` or `Cannot establish any listening sockets` inside an agent sandbox, retry the same smoke launcher with permission to access its isolated local X11 socket. Run it as your normal user. The sandbox can restrict socket access even when Xvfb is installed and available on PATH.

## Project documents

- [Game and opponent](docs/foundation/product.md)
- [Competitive rules and verification](docs/foundation/rules.md)
- [Expansions, coaches, parks and promos](docs/foundation/expansions.md)
- [Components, card custody and coverage](docs/foundation/content.md) and [per-card implementation ledger](docs/foundation/card-content.csv)
- [Design and native authoring](docs/foundation/design.md)
- [Shared code, source inspection and migration order](docs/foundation/engineering.md)
- [Rules sources and rulings](docs/foundation/sources.md)
- [Repository constitution](docs/principles/principles-for-repos.md)

See [the starter-card decision](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/1), [ongoing card coverage](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/2), and [the first playable](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/3). Original project code is MIT licensed. See [LICENSE](LICENSE). Game assets, publisher PDFs and Workshop source stay in owner-local storage. Matt Pocock’s skills carry their [upstream license](.claude/skills/LICENSE).
