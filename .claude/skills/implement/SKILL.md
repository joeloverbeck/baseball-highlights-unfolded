---
name: implement
description: "Implement a piece of work based on a spec or set of tickets."
disable-model-invocation: true
---

Implement the work described by the user in the spec or tickets.

If the user passes a ticket reference, fetch it from the issue tracker and state its title before starting. If the reference is ambiguous, ask.

For visual work, start with filenames, diff stats, and targeted excerpts. Reserve tool output for the captures needed to verify the affected behavior.

Before changing a native decision layout, identify the approved components and where they will be reused. For new layouts, follow the alternatives and selection workflow in `docs/foundation/design.md`.

Call the Skill tool with "tdd" where possible, at pre-agreed seams.

Run typechecking regularly, single test files regularly, and the full test suite once at the end. For desktop control changes, use the README smoke command as soon as the path works. Check the changed controls and labels at the default and minimum supported window sizes before review.

Once done, call the Skill tool with "code-review" to review the work.

Commit your work to the current branch.
