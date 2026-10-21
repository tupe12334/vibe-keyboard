# Push lock

`<scratch>` is the session's scratch folder.

PUSH QUEUE (one pusher at a time):
- Before any cargo build or git push: take the lock with
  mkdir <scratch>/push.lock
  If mkdir fails, another agent holds it: wait 60 s and retry (one bounded loop, max 60 min).
- After your push (or build) ends, success or not: rm -rf <scratch>/fs-focused-target/debug/incremental <scratch>/fs-focused-target/llvm-cov-target, then rmdir <scratch>/push.lock.
- Never remove a lock you did not take.
- Disk stays near 5 GiB because of other sessions. With the warm shared dir, push when df >= 4.5 GiB and set VK_MIN_FREE_KIB=4194304 on the git push command. Never use VK_SKIP_DISK_GUARD.
- ALWAYS export CARGO_PROFILE_DEV_DEBUG=0 (no debug info; halves the build dir) on every cargo command and on git push.
- After your push, ALSO rm -rf your own worktree target/ dir (the hook may build there). Never delete another agent's worktree.
- Commit too: the pre-commit hook may also run cargo. Take the lock, and export CARGO_TARGET_DIR (the shared dir) and CARGO_PROFILE_DEV_DEBUG=0, before `git commit` as well as before `git push`.
- OWNER FILE: right after `mkdir push.lock` succeeds, write one line `echo "$$ <your branch> $(date +%s)" > <scratch>/push.lock/owner`. Remove the owner file before `rmdir push.lock` (rmdir needs an empty dir).
- STALE LOCK: while waiting, if push.lock/owner is older than 30 min (`find <scratch>/push.lock/owner -mmin +30`) AND its pid is gone (`kill -0 <pid>` fails), remove it (`rm <scratch>/push.lock/owner; rmdir <scratch>/push.lock`) and take the lock. If the owner file is missing and the lock dir is older than 30 min, treat it as stale too. Never break a lock whose pid is alive.
