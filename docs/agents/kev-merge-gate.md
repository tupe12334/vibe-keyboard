# Kev merge gate: template

`<scratch>` is the session's scratch folder.

Use it for every vibe-keyboard merge gate. Fill each {field} from live data and leave no field empty.
Body: {"state": "<the STATE text below, filled in>", "questions": {"q": {"type": "noul", "instructions": "Should this PR be merged now?", "criteria": {"true": "merge", "false": "hold"}}}}
Send: curl -s -X POST http://127.0.0.1:8009/v1/systemone -H 'content-type: application/json' -d @<file>

STATE:
Solo repo tupe12334/vibe-keyboard. The user is the only maintainer and gave merge decisions to Kev.
PR {number}: {one-line change}. Files: {files}. Runtime code changed: {yes/no}.
Checks: {each check name = result} (read live via gh).
Independent reviewer verdict: {APPROVE/CHANGES} ({comment link}).
Proof: {mutation result / live run / local test output}.
Risk: {what breaks if it is wrong, and how it is undone (revert PR)}.
Dependabot only: lockfile-only {yes/no}; advisories check {result}; upstream changelog {link}.

Rule: merge when noul >= 0.9. Otherwise ask {"type":"choice","instructions":"Merge policy for this PR?","criteria":{"merge_now":"merge now","hold":"hold"}} (criteria must be a map) and merge when merge_now >= 0.8.
Do not ask the user. If both fail, add the missing facts and ask again once.
