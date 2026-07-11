import os
import sys
import time
import shutil
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
    cleanup,
    PREFIX,
)

HERDR = shutil.which("herdr") or "/run/current-system/sw/bin/herdr"
CONFIG_NO_BIND = (
    "[terminal]\n"
    'default_shell = "/bin/bash"\n\n'
    "[keys]\n"
    'prefix = "ctrl+b"\n'
    'detach = "prefix+d"\n'
    'goto = ""\n\n'
    "[experimental]\nallow_nested = true\n"
)
CONFIG_WITH_BIND = CONFIG_NO_BIND.replace(
    'goto = ""\n', 'goto = ""\nswitch_session = "prefix+s"\n'
)


def write_config(root, text):
    with open(os.path.join(root, "config", "herdr", "config.toml"), "w") as f:
        f.write(text)


def attach_and_press(env, session):
    pid, fd = spawn_pty([HERDR, "--session", session], env)
    drain(fd, 3.0)
    before = ps_argv(pid)
    send(fd, PREFIX, settle=0.7)
    send(fd, b"s", settle=1.0)
    send(fd, b"\x1b[B", settle=0.5)
    send(fd, b"\r", settle=2.0)
    time.sleep(1.2)
    after = ps_argv(pid)
    return pid, fd, before, after


def main():
    log(
        "proving: a RUNNING server that lacked switch_session picks it up after reload-config"
    )
    root = "/tmp/hr%d" % os.getpid()
    shutil.rmtree(root, ignore_errors=True)
    os.makedirs(root, exist_ok=True)
    env = make_env(root, CONFIG_NO_BIND)
    log("root: %s" % root)
    pids, fds = [], []
    try:
        log("--- create target session 'delta', detach so its server persists")
        dpid, dfd = spawn_pty([HERDR, "--session", "delta"], env)
        pids.append(dpid)
        fds.append(dfd)
        drain(dfd, 3.0)
        send(dfd, PREFIX + b"d", settle=1.5)
        time.sleep(1.0)

        log(
            "--- start 'gamma' server WITHOUT switch_session bound, prove prefix+s is inert"
        )
        gpid, gfd, gbefore, gafter = attach_and_press(env, "gamma")
        pids.append(gpid)
        fds.append(gfd)
        log("gamma before=[%s] after=[%s]" % (gbefore, gafter))
        inert = "delta" not in gafter
        log("PROBE baseline-unbound-inert: %s" % inert)
        send(gfd, PREFIX + b"d", settle=1.5)
        time.sleep(1.0)

        log(
            "--- add switch_session=prefix+s to config, reload ONLY gamma's running server"
        )
        write_config(root, CONFIG_WITH_BIND)
        renv = dict(env)
        renv["HERDR_SESSION"] = "gamma"
        rc = subprocess.run(
            [HERDR, "server", "reload-config"],
            env=renv,
            timeout=15,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
        )
        log("reload-config rc=%d out=%s" % (rc.returncode, rc.stdout.strip()[:200]))
        log("sessions:\n%s" % session_list(HERDR, env))

        log("--- re-attach gamma, press prefix+s, expect in-place switch to delta")
        g2pid, g2fd, before2, after2 = attach_and_press(env, "gamma")
        pids.append(g2pid)
        fds.append(g2fd)
        log("gamma(reloaded) before=[%s] after=[%s]" % (before2, after2))
        switched = "delta" in after2
        log("PROBE reloaded-server-switches: %s" % switched)

        verdict = "PASS" if (inert and switched) else "FAIL"
        log(
            "\nRELOAD_VERDICT=%s (inert-before=%s, switch-after-reload=%s)"
            % (verdict, inert, switched)
        )
        log("ROOT=%s" % root)
    finally:
        cleanup(HERDR, pids, fds, env, ("delta", "gamma"))


if __name__ == "__main__":
    main()
