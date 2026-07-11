# Session switcher end-to-end tests

Drive the real `herdr` binary in a pseudo-terminal to prove the in-client
session picker (`switch_session`) switches the current client to another
session in place, the tmux `choose-session` workflow.

## Run

```
python3 tests/e2e/session_switcher/switch_test.py   # picker switches alpha -> beta in place
python3 tests/e2e/session_switcher/reload_test.py   # a running server picks up the keybind after reload-config
```

`switch_test.py` honours `E2E_BIND` / `E2E_TRIGGER` to test an alternate
keybinding (default `prefix+s` / `s`).

## Signal

The definitive assertion is that the client's **same pid** re-execs its argv
from `--session alpha` to `--session beta` (an in-place switch, no new
process, no detach to a shell). Screen scraping of the picker text is a weak
secondary signal and is not required to pass.

## Notes

- Roots the isolated session tree under a short `/tmp/he<pid>` path. macOS caps
  unix-domain socket paths at ~104 bytes, and the default `$TMPDIR`
  (`/var/folders/...`) plus `sessions/<name>/herdr.sock` overflows it, so the
  background server silently fails to bind and the client times out.
- The background server must be allowed to daemonize; run outside any process
  sandbox that reaps detached children.
