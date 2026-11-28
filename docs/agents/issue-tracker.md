# Work tracking

Use GitHub Issues and PRs in `joeloverbeck/baseball-highlights-unfolded`. Publish specs and tickets as issues. Link each spec, its source audit, blockers, evidence, owner decisions and merged revisions. Keep per-card coverage in [the card ledger](../foundation/card-content.csv). Keep private artwork outside issue and PR attachments. The owner separately activates implementation after readiness.

When pausing work in a local checkout, record its path, branch, base and current commit SHAs, and whether it has uncommitted edits in the existing handoff. Include the owning clone when the worktree belongs to another clone. If a subagent choice is pending, repeat the proposed job, recommended model and effort, and the decision needed. Check the location and Git status when resuming.

For a checkpoint bundle, add its path, SHA-256 digest, and inventory of modified and untracked files to the same handoff. Account for ignored files separately. Before removing a source checkout, compare its current changes and local-only files with the retained bundle or backup.

## Draft publication

When a spec will use a draft PR, check publication authority at activation. If it is missing, ask for permission to push the named integration branch and open a draft in the target GitHub repository. Local implementation can continue while permission is pending. Before merging, check that the owner's authorization covers the reviewed commit, evidence, and target branch. Use existing authorization when it covers the action.

## GitHub CLI

- Check the target before writing with `gh repo view --json nameWithOwner`.
- If the GitHub connector returns `403 Resource not accessible by integration` for an authorized write, use the authenticated `gh` CLI for that write. For PRs, pass `--repo` and use `--body-file` for the description.
- Read a ticket and its discussion with `gh issue view <number> --json number,title,body,labels,comments`. Find work with `gh issue list --state open` and the relevant label filters.
- Publish with `gh issue create --title "..." --body-file <file>`. Use a body file for multiline comments with `gh issue comment <number> --body-file <file>`.
- Use native sub-issues and dependencies. Create with `--parent <parent>` and `--blocked-by <blocker>` as applicable. Link existing issues with `gh issue edit <parent> --add-sub-issue <child>` or `gh issue edit <child> --add-blocked-by <blocker>`. Use `gh issue view <child> --json blockedBy` to inspect blockers and check their state.
- Change labels with `gh issue edit <number> --add-label <label>` or `--remove-label <label>`. To close an issue, record the reason in a comment, then run `gh issue close <number>`.

**PRs as a request surface: no.**

Wayfinder uses a GitHub map issue and native child issues. Follow its map and ticket label conventions. Use native dependencies for the frontier and `gh issue edit <number> --add-assignee @me` for a claim within authorized scope.
