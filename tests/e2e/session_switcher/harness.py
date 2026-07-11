import os
import pty
import select
import struct
import fcntl
import termios
import time
import subprocess
import signal
import sys

COLS, ROWS = 120, 40
PREFIX = b"\x02"


def log(msg):
    sys.stdout.write(str(msg) + "\n")
    sys.stdout.flush()


def make_env(root, config_text):
    env = {
        k: v
        for k, v in os.environ.items()
        if not k.startswith("HERDR") and k not in ("TMUX", "VSCODE_PID")
    }
    for sub in ("config", "data", "state", "cache"):
        env["XDG_%s_HOME" % sub.upper()] = os.path.join(root, sub)
        os.makedirs(os.path.join(root, sub), exist_ok=True)
    env["HOME"] = root
    env["TERM"] = "xterm-256color"
    env["HERDR_LOG"] = "debug"
    cfgdir = os.path.join(root, "config", "herdr")
    os.makedirs(cfgdir, exist_ok=True)
    with open(os.path.join(cfgdir, "config.toml"), "w") as f:
        f.write(config_text)
    return env


def spawn_pty(args, env):
    pid, fd = pty.fork()
    if pid == 0:
        try:
            os.execvpe(args[0], args, env)
        except Exception as exc:
            os.write(2, ("execvpe failed: %s\n" % exc).encode())
            os._exit(127)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
    return pid, fd


def drain(fd, seconds):
    end, out = time.time() + seconds, b""
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.2)
        if not r:
            continue
        try:
            chunk = os.read(fd, 65536)
        except OSError:
            break
        if not chunk:
            break
        out += chunk
    return out


def send(fd, data, settle=0.6):
    os.write(fd, data)
    return drain(fd, settle)


def ps_argv(pid):
    try:
        return subprocess.check_output(
            ["ps", "-o", "command=", "-p", str(pid)], text=True
        ).strip()
    except Exception:
        return ""


def session_list(herdr, env):
    return subprocess.run(
        [herdr, "session", "list"],
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=10,
    ).stdout


def switch_logged(root):
    logdir = os.path.join(root, "state", "herdr")
    if not os.path.isdir(logdir):
        return False
    for name in os.listdir(logdir):
        if not name.startswith("herdr.log"):
            continue
        try:
            with open(os.path.join(logdir, name), errors="replace") as lf:
                if "client session switch requested via keybind" in lf.read():
                    return True
        except Exception:
            pass
    return False


def cleanup(herdr, pids, fds, env, names):
    for fd in fds:
        try:
            os.close(fd)
        except Exception:
            pass
    for pid in pids:
        try:
            os.kill(pid, signal.SIGKILL)
        except Exception:
            pass
    for name in names:
        subprocess.run(
            [herdr, "session", "stop", name],
            env=env,
            timeout=10,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
