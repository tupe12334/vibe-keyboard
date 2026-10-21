# PR governor prompt: template

`<scratch>` is the session's scratch folder.

Fill in {PR}, {SUMMARY}, {CHECKS_TO_REVIEW}, {WORKTREE}, {BRANCH}. Give it to a Haiku agent as its whole prompt.

Govern PR https://github.com/tupe12334/vibe-keyboard/pull/{PR} ({SUMMARY}). Pass `-R tupe12334/vibe-keyboard` to every gh call.
1. Do a read-only review of `gh pr diff {PR}`. Check {CHECKS_TO_REVIEW}, and check that nothing unrelated changed. Run any static check that applies (shellcheck, osacompile syntax check, plutil -lint, yaml.safe_load) on scratch copies under `<scratch>/`. Post a PR comment that starts "Verdict: APPROVE" or "Verdict: CHANGES". GitHub blocks approving your own PR, so post a comment. If the verdict is CHANGES, stop and report.
2. Run the Kev gate exactly as docs/agents/kev-merge-gate.md says.
3. Run `gh pr merge {PR} -R tupe12334/vibe-keyboard --squash --delete-branch`. Confirm the state reads MERGED and record the merge SHA. Never create tags or releases.
4. Remove the worktree with `git -C /Users/ofek/dev/git/github/tupe12334/vibe-keyboard worktree remove {WORKTREE}`, without --force. If untracked files block it, rm -rf its target/ first. Then delete local branch {BRANCH}.
Never use --no-verify. Never use git stash. Never touch other PRs. Report: the verdict, the Kev scores and the merge SHA.

Runtime-change PRs (app code, not CI or docs): before step 3, also prove the change in the real app (install, launch from the shell, check app.log), or hold the PR and report why.

Editor proof (any PR that changes the editor, apps/desktop/**):
1. Take the push lock (docs/agents/push-lock.md). Check df: at least 7 GiB must be free. Run `shasum ~/.config/vibe-keyboard/profile.toml` and record the result.
2. Build the PR branch with build-app.sh, using the shared CARGO_TARGET_DIR. Kill the app only by PID.
3. Launch `VK_OPEN_EDITOR=1 nohup "$HOME/Applications/Vibe Keyboard.app/Contents/MacOS/vibe-keyboard-desktop" >/dev/null 2>&1 & disown`. Wait 8 s. Run `screencapture -x <scratch png>` and Read the PNG.
4. PASS needs all four: the editor window shows the user's real keys and pages (not "Loading…", not blank); app.log has no new "editor webview" warning; app.log has no "needs Accessibility" line; the profile.toml hash is unchanged. Click nothing in the editor.
5. Afterwards, reinstall origin/main unless the PR merged. Relaunch without the flag. Clean the release, incremental and llvm-cov dirs. Release the lock.

Release-workflow PRs (any change to .github/workflows/release.yml or .github/actions/**):
- The PR body must hold the trigger truth table: tag push → publish; schedule → dry; pull_request → dry; dispatch dry_run=true → dry; dispatch dry_run=false → publish.
- Check each row against the YAML: read every publishing step's `if:` and the DRY_RUN expression. One wrong row means Verdict: CHANGES.
- Proof: a dry run on the PR branch passes, and `gh release list` and the remote tags are unchanged.

Signing: run every build-app.sh with VK_SIGN_IDENTITY='Apple Development: Ofek Gabay (HN724A92RQ)'. The default identity is not in the keychain, so builds fell back to ad-hoc signing, which changes the app identity on every build. After the install, check app.log for "needs Accessibility" and report it.
