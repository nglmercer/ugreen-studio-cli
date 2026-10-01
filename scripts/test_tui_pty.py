#!/usr/bin/env python3
"""Linux-only offline terminal lifecycle smoke tests. Never connects to Bluetooth."""
import argparse
import errno
import fcntl
import os
import pty
import select
import signal
import struct
import subprocess
import sys
import termios
import time
from pathlib import Path


def drain(master, capture, duration):
    deadline = time.monotonic() + duration
    while time.monotonic() < deadline:
        ready, _, _ = select.select([master], [], [], max(0, deadline - time.monotonic()))
        if not ready:
            return
        try:
            data = os.read(master, 65536)
        except OSError as exc:
            if exc.errno == errno.EIO:
                return
            raise
        if not data:
            return
        capture.extend(data)


def resize(slave, rows, columns):
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, 0, 0))


def scenario(binary, name, args, actions):
    master, slave = pty.openpty()
    proc = None
    output = bytearray()
    try:
        resize(slave, 30, 110)
        before = termios.tcgetattr(slave)
        proc = subprocess.Popen([str(binary), *args], stdin=slave, stdout=slave,
                                stderr=slave, env={**os.environ, 'TERM': 'xterm-256color'})
        drain(master, output, 0.5)
        if b'\x1b[?1049h' not in output:
            raise AssertionError(f'{name}: no alternate-screen entry: {bytes(output)!r}')
        for action in actions:
            if isinstance(action, tuple):
                resize(slave, *action)
                proc.send_signal(signal.SIGWINCH)
            else:
                os.write(master, action)
            drain(master, output, 0.25)
        try:
            code = proc.wait(timeout=5)
        except subprocess.TimeoutExpired as exc:
            raise AssertionError(f'{name}: process did not exit promptly') from exc
        drain(master, output, 0.1)
        if code != 0:
            raise AssertionError(f'{name}: exit {code}: {bytes(output[-3000:])!r}')
        if termios.tcgetattr(slave) != before:
            raise AssertionError(f'{name}: terminal attributes were not restored')
        if b'\x1b[?1049l' not in output:
            raise AssertionError(f'{name}: no alternate-screen exit')
        if b'panicked at' in output:
            raise AssertionError(f'{name}: unexpected panic')
        print(f'PASS {name}: exit=0, terminal restored, {len(output)} output bytes')
    finally:
        if proc is not None and proc.poll() is None:
            proc.kill()
            proc.wait(timeout=5)
        os.close(master)
        os.close(slave)


def main():
    if not sys.platform.startswith('linux'):
        raise SystemExit('This PTY smoke test is supported on Linux only.')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('binary', type=Path, help='Default-feature ugreen release/debug executable')
    binary = parser.parse_args().binary.resolve(strict=True)
    scenarios = [
        ('explicit quit', ['tui'], [b'q']),
        ('automatic TTY entry', [], [b'q']),
        ('Ctrl+C exit', ['tui'], [b'\x03']),
        ('small-screen resize recovery', ['tui'], [(8, 24), b'j', (30, 110), b'q']),
        ('address edit cancellation', ['tui'], [b'a', b'AA:BB:CC:DD:EE:FF', b'\x1b', b'q']),
    ]
    for name, args, actions in scenarios:
        scenario(binary, name, args, actions)
    print(f'{len(scenarios)} offline PTY scenarios passed; no Bluetooth actions requested.')


if __name__ == '__main__':
    main()
