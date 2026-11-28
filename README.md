# Baseball Highlights: 2045 Unfolded

An Ubuntu-first native Rust tabletop game in development. Manage a team through a competitive series against Codex or Claude Code. Make the lineup, tactical and buying decisions, talk baseball with the opposing manager, and watch the consequences unfold in a 3D ballpark.

The native review opens the Expert Buy Round and follows its purchases into the opening of game two. See the [first playable specification](docs/specs/two-expert-games.md) for the full series work. The [original-card audit](docs/foundation/content.md#starter-card-proxies) records approved equivalents for eight starter faces. All four original teams can be represented with the available artwork.

## Open the reviewed Buy Round

On Ubuntu with Rust 1.100 and a Vulkan adapter, run `cargo run --locked` from the checkout. The game reads the audited collection from `~/Pictures/Baseball Highlights: 2045 Resources`. Use `-- --assets DIRECTORY` for another copy of that extraction. The game reports any missing or changed required face before opening the table.

SF earns $6 and NY earns $7. In the SF manager seat, choose **SF buys first** or **NY buys first**. To replay the reviewed round, choose SF, select **Frankie Foxx**, then **Sign Frankie Foxx for $4**. SF keeps $2 and Mickey Maris takes Foxx’s slot. Select an In Play player, then **Send to the Minor Leagues** to return SF to 15 active players. Select **Finish buying** to pass the turn.

Open the same save with `--seat ny`, select **Mickey Maris**, then **Sign Mickey Maris for $7**. Babe Bench takes the slot. Choose NY’s demotion and select **Finish buying**. Both Dugouts display their remaining In Play cards face up, each roster has 15 active players, and any unspent revenue expires. **Round history** shows both managers’ choices. Press Space to finish any card motion.

The default save is `baseball-highlights-unfolded/expert-buy-round.json` under `$XDG_STATE_HOME` or `~/.local/state`. Use `-- --save FILE` with a fresh path to start at the purchase-order choice. Two desktop windows using the same save can follow each other’s turns. Close and relaunch to resume whichever choice is pending. For an isolated agent seat, add `--agent-seat sf` or `--agent-seat ny` after `--`. The seat serves the shared JSON-lines protocol over standard input and output.

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

In SF choose **SF buys first**, then inspect **Peewee Rizzuto** and **Frankie Foxx**. Both cost $4. The details above the face give speed, revenue and PH. Sign Foxx and check that Mickey Maris fills slot 5 and SF has $2 left. Inspect **ST-004**, confirm **Send to the Minor Leagues**, then select **Finish buying**. In NY sign Maris, check that Babe Bench fills the slot, demote **ST-056**, and finish buying. Both active rosters return to 15.

In SF select **Draw hands for game two**. NY hosts and leads the series 1–0. Select **ST-013**, then **Play Veteran Natural / ST-013**. In NY select **FA-067**, then **Play Mickey Maris / FA-067**. Maris cancels SF’s single and threatens a single and a home run. In SF select **FA-065**, then **Play Frankie Foxx / FA-065**. Preview both Glove targets. Cancelling box 1 gives NY one run and leaves the bases empty. Cancelling box 2 keeps the game score 0–0 and puts Maris on first as an Average runner. Select **Commit Glove / Cancel box 2** to see that runner in both windows, with Foxx’s double waiting at home. The review covers these opening plays from hand.

## Review the Glove decision

Run `cargo run --locked -- --review-glove` to open the high field camera as SF. NY leads 2–1 with an average runner on second. Select **Preview box 1 / Single** or **Preview box 2 / Home run** to compare the effects while the live field holds. Select **Commit Glove / Cancel box 1** for NY 4–SF 1 with empty bases, or **Commit Glove / Cancel box 2** for NY 3–SF 1 with Mickey Maris on first. In either case, SF’s single then waits at home. The scorecard lists the committed events in order.

Close and reopen the same save to resume. Add `--seat ny` in a second window to follow the public field as NY. The review has its own `glove-review.json` in the same state directory as the Buy Round. Use `--review-glove --save FILE` with a fresh path for the other outcome. The review ends after this play.

## Repository checks

CI runs `cargo fmt --check`, the Buy Round rules and save tests with `cargo test --locked --no-default-features`, and `cargo check --locked --all-targets` for the Ubuntu app. With Python 3.10+ and Pillow installed, run `python3 -B -m unittest discover -s tests -v` for the original-set ledger and design restore checks.

Run `scripts/native-buy-round` on Ubuntu with Xvfb, xauth, xdotool, Pillow with X11 capture support, and the audited owner-local card collection. It uses the normal desktop binary on an isolated display to exercise primary and secondary clicks, both signings and refills, close and resume, and the face-down piles and Dugouts. It prints the adapter and the temporary directory containing captures, logs, and the scratch save. Use `--assets DIRECTORY` for another copy of the collection or set `WGPU_ADAPTER_NAME` to select an available Vulkan adapter.

Run `scripts/native-buy-round --game-two` for the tracked Foxx and Maris replay through two JSON-lines seats and two ordinary X11 windows. The command checks both Glove previews and compares the final saves byte for byte. It also exercises Maris's play at 1080×720. The printed evidence directory contains the control trace, captures, scratch saves and SHA-256 digests for the executable and starting position.

Run `scripts/native-glove` with the same prerequisites to exercise both Glove commitments, switching previews, the NY observer and reopening the pending SF choice.

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
