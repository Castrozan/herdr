import os
import sys
import time
import shutil
import tempfile
import subprocess

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from harness import (
    log,
    make_env,
    spawn_pty,
    drain,
    send,
    ps_argv,
    session_list,
    switch_logged,
    cleanup,
    PREFIX,
)

HERDR = shutil.which("herdr") or "/run/current-system/sw/bin/herdr"
BIND = os.environ.get("E2E_BIND", "prefix+s")
TRIGGER = os.environ.get("E2E_TRIGGER", "s").encode()

CONFIG = (
    "[terminal]\n"
    'default_shell = "/bin/bash"\n'
    'shell_mode = "login"\n\n'
    "[keys]\n"
    'prefix = "ctrl+b"\n'
    'detach = "prefix+d"\n'
    'goto = ""\n'
    'switch_session = "%s"\n\n'
    "[experimental]\nallow_nested = true\n"
) % BIND


def main():
    log("herdr: %s" % HERDR)
    log(
        "version: %s" % subprocess.check_output([HERDR, "--version"], text=True).strip()
    )
    log("binding switch_session=%r, trigger byte=%r" % (BIND, TRIGGER))
    root = "/tmp/he%d" % os.getpid()
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(root, exist_ok=True)
    env = make_env(root, CONFIG)
    log("root: %s" % root)
    pids, fds = [], []
    try:
        log("--- step 1: create target session 'beta', detach so its server persists")
        bpid, bfd = spawn_pty([HERDR, "--session", "beta"], env)
        pids.append(bpid)
        fds.append(bfd)
        drain(bfd, 3.0)
        send(bfd, PREFIX + b"d", settle=1.5)
        time.sleep(1.0)
        listing = session_list(HERDR, env)
        log("session list:\n%s" % listing)

        log("--- step 2: attach 'alpha'")
        apid, afd = spawn_pty([HERDR, "--session", "alpha"], env)
        pids.append(apid)
        fds.append(afd)
        screen = drain(afd, 3.0)
        log("alpha pid=%d argv_before=[%s]" % (apid, ps_argv(apid)))

        log("--- step 3: send prefix (ctrl+b) then trigger %r" % TRIGGER)
        screen += send(afd, PREFIX, settle=0.7)
        screen += send(afd, TRIGGER, settle=1.2)
        blob = screen.decode("utf-8", "replace").lower()
        hits = [m for m in ("switch session", "no other sessions", "beta") if m in blob]
        log("PROBE picker-opened markers=%s" % hits)

        log("--- step 4: select target (Down, Enter), verify in-place re-exec")
        send(afd, b"\x1b[B", settle=0.6)
        send(afd, b"\r", settle=2.0)
        time.sleep(1.5)
        argv_after = ps_argv(apid)
        log("alpha pid=%d argv_after=[%s]" % (apid, argv_after))
        switched = "beta" in argv_after
        logged = switch_logged(root)
        log("PROBE switch-log-line=%s" % logged)
        log("PROBE switched-in-place=%s" % switched)
        with open(os.path.join(root, "alpha_screen.bin"), "wb") as sf:
            sf.write(screen)
        verdict = "PASS" if (switched or logged) else "FAIL"
        log("\nE2E_VERDICT=%s" % verdict)
        log("E2E_ROOT=%s" % root)
    finally:
        cleanup(HERDR, pids, fds, env, ("beta", "alpha"))


if __name__ == "__main__":
    main()
