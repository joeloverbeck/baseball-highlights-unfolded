# Baseball Highlights: 2045 Unfolded

An Ubuntu-first native Rust tabletop game in development. Manage a team through a competitive series against Codex or Claude Code. Make the lineup, tactical and buying decisions, talk baseball with the opposing manager, and watch the consequences unfold in a 3D ballpark.

The first native increment opens the reviewed Expert Buy Round. See the [first playable specification](docs/specs/two-expert-games.md) for the full series work. The [original-card audit](docs/foundation/content.md#starter-card-proxies) records approved equivalents for eight starter faces. All four original teams can be represented with the available artwork.

## Open the reviewed Buy Round

On Ubuntu with Rust 1.100 and a Vulkan adapter, run `cargo run --locked` from the checkout. The game reads the audited collection from `~/Pictures/Baseball Highlights: 2045 Resources`. Use `-- --assets DIRECTORY` for another copy of that extraction. The game reports any missing or changed required face before opening the table.

In the SF manager seat, select **Frankie Foxx**, then **Sign Frankie Foxx for $4**. SF keeps $2 and Mickey Maris takes Foxx’s slot. Select an In Play player, then **Send to the Minor Leagues** to return SF to 15 active players. Press Space to finish any card motion. Close and relaunch with the same save to resume a pending demotion.

The default save is `baseball-highlights-unfolded/expert-buy-round.json` under `$XDG_STATE_HOME` or `~/.local/state`. Use `-- --save FILE` to start a separate series in a fresh file. For an isolated agent seat, add `--agent-seat sf` or `--agent-seat ny` after `--`. The seat serves the shared JSON-lines protocol over standard input and output. NY observes the SF decision from the same save. The increment ends at SF’s demotion.

## Repository checks

CI runs `cargo fmt --check`, the Buy Round rules and save tests with `cargo test --locked --no-default-features`, and `cargo check --locked --all-targets` for the Ubuntu app. With Python 3.10+ and Pillow installed, run `python3 -B -m unittest discover -s tests -v` for the original-set ledger and design restore checks.

Run `scripts/native-buy-round` on Ubuntu with Xvfb, xauth, xdotool, Pillow with X11 capture support, and the audited owner-local card collection. It uses the normal desktop binary on an isolated display to exercise primary and secondary clicks, the public refill, close and resume, and the visible face-down piles. It prints the adapter and the temporary directory containing captures, logs, and the scratch save. Use `--assets DIRECTORY` for another copy of the collection or set `WGPU_ADAPTER_NAME` to select an available Vulkan adapter.

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
