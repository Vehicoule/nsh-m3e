# 0014. Merge the NSH bump pull request automatically when every check is green

- Status: Accepted
- Date: 2026-11-06
- Deciders: the user

## Context

[ADR 0006](0006-pinned-versions-and-dependencies.md) has `.github/workflows/nsh-bump.yml` open one pull request a day
that moves the NSH pin, and says "CI runs on it; the user merges it when green". The first one, #26, waited with its
CI green, and `.github/CODEOWNERS` (`*` to the owner) asked the owner for a review GitHub does not require.

On 2026-11-06 the user asked that the bot merge it on its own when CI is green ("ba c'est censé le faire de lui
meme"), and only the bump pull requests ("attention merge automatique pour les bumps").

Facts:

- The bot app (`vehicoule-bot`) opens the bump with Contents and Pull requests write; a push made with its token runs
  the workflows of the push, where one made with `GITHUB_TOKEN` does not.
- GitHub's own auto-merge needs a repository setting and required checks on `main` to wait for; without required
  checks it merges at once, before CI. Neither is in the repository's files.
- A red bump is how ADR 0006 sees which NSH change breaks the library: its fix goes in the bump pull request.

## Options

1. **A workflow merges the bump when every check on its head is green.** `nsh-bump-merge.yml` runs when `CI` (and,
   with ADR 0013's screenshots, `Visual checks`) completes on the `nsh-bump` branch, and merges the open bump at that
   commit with the bot's token if no check run on it is still running or other than success, skipped or neutral.
   Cost: one workflow. Gives: the merge waits for every check whatever the repository settings, and a red bump stays
   open.
2. **GitHub's auto-merge** (`gh pr merge --auto` in `nsh-bump.yml`). Cost: needs "Allow auto-merge" and required
   status checks on `main`, set by hand in the repository settings, which no file shows or checks. Gives: no new
   workflow.
3. **Keep the merge by hand** (ADR 0006 as it is). Not what the user asked.

## Decision

Option 1. It changes ADR 0006 on one point: the bump is merged by `nsh-bump-merge.yml` when every check is green, not
by the user. No other pull request is merged by it: the workflow merges only an open pull request from the bot, on
the `nsh-bump` branch of this repository, at the commit whose checks completed.

## Consequences

- The pin follows NSH's `main` with no one in the loop while the library builds and its checks pass; a red bump waits
  for its fix, as before.
- `CODEOWNERS` still asks the owner for a review on each bump; it is not required, and the bump merges without it.
- A workflow added later that checks pull requests is added to `nsh-bump-merge.yml`'s list. The merge waits for
  every check run on the commit, listed or not; but if an unlisted workflow is the last to complete, nothing runs the
  merge after it, and the bump waits for the next day's run.
