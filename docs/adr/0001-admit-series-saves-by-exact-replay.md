# Admit series saves by exact replay of manager choices

On 2026-11-28 the owner selected one current Baseball save format for the Expert Buy Round through the opening of game two and the standalone Glove review. The [development save policy](../foundation/engineering.md#series-saves-during-development) governs compatibility and its review point.

Save the series identity, named starting setup, private seed, ordered manager choices and the committed position they produced. Reconstruct the setup and apply each choice through the rules used in live play. Admit the save only if every choice was legal and the complete position matches, including card instances, hidden pile order and the pending decision. The scorecard records the resulting play. A signing is one manager choice even though charging revenue, moving a player and revealing the refill all follow from it. Keep the ordered choices explicit so that changing a scorecard event never requires reverse-engineering a manager's intent.

An explicit recovery takes the previous committed record to an unused destination. Apply the same format, replay and launch-mode checks before creating it. Preserve the active and previous files on refusal and successful recovery alike. The recovered series continues under its own write baseline. The desktop and agent seats share the same rules and persistence entry points.

## Evidence for the implementation spec

The [implementation spec and acceptance tests](https://github.com/joeloverbeck/baseball-highlights-unfolded/issues/22) are tracked under the two-game series.

The first consumer is a manager returning to a pending demotion or Glove choice in the [two-game series](../specs/two-expert-games.md). This refactor retires the v1 conversion before more rules have to accommodate it. Preserve both reviewed paths by regenerating same-format scratch saves from their known setups. Then return to the two-game work.

| Inspected checkout | Save path and consequence |
| --- | --- |
| Baseball `af8cb886` | `SeriesFormat::validate` reconstructs a start and extracts actions from a mixed event history. `LegacyPosition::restore` additionally reconstructs a superseded Buy Round and adjusts its hidden refill order. Retire that reader, its conversion fixture and migration assertion. Keep the same-build pending-demotion case. |
| unfolded-tabletop `443ffe13` and Baseball's pin `1d1907d4` | The record-replacement package is identical at these revisions. Its save envelope, checksum, baseline, locked replacement and recovery into a fresh path already accept a game-owned validator. |
| Arkham `cf7d92df` | Gathering's `Session::validate` recreates its seeded origin, calls `choose` at each reconstructed revision and compares the resulting session. A named review origin can take the same validation path as ordinary play. |
| Cuba Libre `190ff9ac` | `Payload::replay` starts from the seed, applies recorded decisions and compares the resulting game. `Session::open` in the save adapter supports explicit previous-record recovery. Its ADR 0003 records why several special-purpose admission checks were retired. |
| Netrunner `5727b703` | Its save adapter validates a snapshot through `resume` and admits a recovered record for the requested game content before creating a destination. This consumer needs the shared storage contract with its own game validation. |
| Mudsong `9630c8c0` | Its rules tests exercise repeatable draws and shuffles. No ordinary match-save consumer was identified for this change. |

Implement in Baseball using its existing library pin. There is no producer change or sibling migration in the selected design. If implementation uncovers genuinely common code that needs extraction, follow the producer and affected-consumer merge, pull and verification order in the [engineering foundation](../foundation/engineering.md) before dependent Baseball changes.

Exercise create, submit, observe, refresh, reopen and explicit recovery through the game entry points used by the two seats. Compare both seats at a pending decision, commit it once and verify the result after reopening. Check competing writes and refusals against the original file contents. Reseal a changed position with a valid envelope checksum to demonstrate the independent replay check. Exercise a foreign format, wrong review mode, unusable previous record and occupied recovery destination without overwriting evidence. Repeat the existing native Buy Round/game-two and Glove routes after restart. Keep their fixed card identities, hidden information and observable outcomes while allowing their serialized fixtures to change.

The source investigation and existing headless tests establish the baseline. Verification of the replacement and its native entry points belongs to implementation. Consulted PURP-3, WORK-1–3/5/6, EVID-1–5, AUTH-1–3, AGENT-5, OPS-1/4, GOV-1, ADOPT-1/2, INT-1 and LONG-1.
