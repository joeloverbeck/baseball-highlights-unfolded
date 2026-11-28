# Work tracking

Use GitHub Issues and PRs in `joeloverbeck/baseball-highlights-unfolded`. Publish specs and tickets as issues. Link each spec, its source audit, blockers, evidence, owner decisions and merged revisions. Keep per-card coverage in [the card ledger](../foundation/card-content.csv). Keep private artwork outside issue and PR attachments. The owner separately activates implementation after readiness.

When pausing work in a local checkout, record its path, branch, base SHA, and whether it has uncommitted edits in the existing handoff. Include the owning clone when the worktree belongs to another clone. Check that location and its Git status when resuming.

## GitHub CLI

- Check the target before writing with `gh repo view --json nameWithOwner`.
- Read a ticket and its discussion with `gh issue view <number> --json number,title,body,labels,comments`. Find work with `gh issue list --state open` and the relevant label filters.
- Publish with `gh issue create --title "..." --body-file <file>`. Use a body file for multiline comments with `gh issue comment <number> --body-file <file>`.
- Use native sub-issues and dependencies. Create with `--parent <parent>` and `--blocked-by <blocker>` as applicable. Link existing issues with `gh issue edit <parent> --add-sub-issue <child>` or `gh issue edit <child> --add-blocked-by <blocker>`. Use `gh issue view <child> --json blockedBy` to inspect blockers and check their state.
- Change labels with `gh issue edit <number> --add-label <label>` or `--remove-label <label>`. To close an issue, record the reason in a comment, then run `gh issue close <number>`.

**PRs as a request surface: no.**

Wayfinder uses a GitHub map issue and native child issues. Follow its map and ticket label conventions. Use native dependencies for the frontier and `gh issue edit <number> --add-assignee @me` for a claim within authorized scope.
