# Baseball Highlights: 2045 Unfolded

An Ubuntu-first native Rust tabletop game in development. Manage a team through a competitive series against Codex or Claude Code. Make the lineup, tactical and buying decisions, talk baseball with the opposing manager, and watch the consequences unfold in a 3D ballpark.

This repository currently contains the requirements and [first playable specification](docs/specs/two-expert-games.md). The [original-card audit](docs/foundation/content.md#starter-card-proxies) records approved equivalents for eight starter faces. All four original teams can be represented with the available artwork.

## Repository checks

With Python 3.10+ and Pillow installed, run `python3 -B -m unittest discover -s tests -v` from the repository root. CI runs the same original-set ledger and design restore checks on public inputs.

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
