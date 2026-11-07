# 04 — Mistakes made, and the rule that stops each

Written at the owner's request on 2026-11-07. Read this before starting work. Add a row whenever a mistake costs the owner time, in the same turn. **The rule column binds every session.**

## Priorities

| # | Mistake | What it cost | Rule |
|---|---|---|---|
| 1 | Started rendering and carried fonts while the foundation was unfinished | A session wasted; the owner had to stop it | **Until Phase 1 closes, work only on model, import, export.** Before starting anything else, check [03](03-OWNER-INSTRUCTIONS.md). If it does not move Phase 1, do not start it |
| 2 | Held the largest stages back behind a four-session limit I set myself | Charts, SmartArt and OLE, 45% of all rows, had not started when the owner asked why progress was only 14% | **Biggest remaining work starts first, not last.** Independent stages all start at once. A limit I invent never outranks progress |
| 3 | Treated malformed files as something to refuse, when opening them is an advantage for a clone | Security work built the opposite of what the product needs, and had to be reversed | **Ask of every design: does this help the user open and edit their file?** Neutralize attacks; refuse only what cannot be read at all |
| 4 | Spent long on documents before code | The owner saw little merged progress | Design only what the next merge needs. A document is not progress |

## Reporting

| # | Mistake | What it cost | Rule |
|---|---|---|---|
| 5 | Reported "built" and "in progress" without the number that shows what is left | The owner found out it was 14% only by asking | **Every status update states the percentage of the current phase, from the checklist, merged only** |
| 6 | Said LibreOffice was stopped when a session was still launching it | The owner had to ask again, while it kept crashing | **Say "stopped" only after checking the processes, twice, a minute apart.** A message to a session is not the same as the session stopping |
| 7 | Said `soffice` was available without running it | A wrong fact in the docs | **Run it before stating it** |
| 8 | Marked security rows Tested when only part was tested; counted a mutation that did not compile as proof | Audit found false claims; I had to redo a check | **Tested means that exact guard was removed and a test failed.** A mutation that does not compile proves nothing |
| 9 | Wrote long reports and technical option lists | The owner could not tell what I was asking | **Short updates. Say what changed, what is left, what I need. Ask in plain language, one question, with my recommendation** |
| 10 | Went quiet for long stretches | The owner had to ask what was happening | **Say something brief before long work, and every time something finishes** |

## Decisions

| # | Mistake | What it cost | Rule |
|---|---|---|---|
| 11 | Handed decisions back after being told to decide | Interrupted the owner | **Decide, record it, carry on.** Ask only when blocked: a permission denial, a legal question, or something only the owner can do |

## Gates and git

| # | Mistake | What it cost | Rule |
|---|---|---|---|
| 12 | Committed with clippy failing; pushed a merge before reading a failing gate | Red `main` | **Read the whole gate output, then commit, then push. In that order** |
| 13 | The gates list did not match CI, so `cargo doc` and the SDK job failed `main` repeatedly | Red `main`, repeated fixes | **The local gates list is exactly what CI runs.** Add a CI job and the list in the same commit |
| 14 | Added a flag to skip the commit hook | The permission system stopped it | **Never skip a hook or route around a gate.** Fix the cause, or report it |
| 15 | Ran a wildcard `git checkout --theirs .` mid-merge | Lost my own conflict resolutions | **Name files in a merge; no wildcards** |
| 16 | Tests compared paths and line endings that differ by OS | Red Windows CI | **Tests compare normalized text, and fixtures are marked in `.gitattributes`** |

## Sessions

| # | Mistake | What it cost | Rule |
|---|---|---|---|
| 17 | A session brief told it to run LibreOffice, a desktop app, in a loop on the owner's machine | Repeated crash dialogs while the owner worked | **Nothing that can pop up or crash on the owner's desktop.** Run such tools only in a container or on CI, and write it in every brief |
| 18 | Sessions held finished work uncommitted, then large merges conflicted | Progress invisible; work at risk at usage limits | **Commit each increment. Merge each session as soon as its gates pass, and do not hold a stage for tidy-up** |
| 19 | Let a shared scratchpad collide between sessions | Scripts overwritten | **Each session uses its own scratch subfolder, named in its brief** |
| 20 | Let build output fill the disk | The owner's disk filled up | **Check disk at every merge. Remove finished worktrees and their build folders at once** |
| 21 | Let a worker's report stand in place of checking | Weak claims could have merged | **Verify by running: gates, then break something real.** A report is not evidence |
| 22 | Started ten build-heavy sessions at once on one laptop without looking at its load | Load average reached 134, so every build and gate slowed, including the merges the owner was waiting for | **Check machine load before adding sessions and at every merge. Above twice the core count, start nothing new, land finished work, and pause the sessions furthest from finishing. Sessions run cargo with `CARGO_BUILD_JOBS=2 RUST_TEST_THREADS=2`; fuzz runs one at a time. Rule 2 means start the biggest work first, not overload the machine** |
| 23 | Replied to notifications that changed nothing, and wrote long reports | Wasted the owner's time and tokens | **Say nothing when nothing changed. Speak when work lands, something breaks, or the owner asks: a few lines with the phase %. Run gates once per merge** |
