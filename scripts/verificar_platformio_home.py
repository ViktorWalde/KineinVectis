"""PIO Home through the real core and the real PlatformIO (0.4.1h4b2b4h).

Needs `pio` (PlatformIO Core 6) in PATH. The core starts PIO Home with
`platformio.homeStart`: the session page must answer 200 and the server root
403, a second start must reuse the same session, `platformio.homeStop` must
close the port, and after the core exits nothing may still listen. No project
is opened and nothing is installed beyond what `pio home` installs itself on
its first run. The core runs with a dead HTTP proxy in its environment: its
readiness check is local and must not go through it (52 §14.45).
"""
import argparse
import json
import os
from pathlib import Path
import queue
import socket
import subprocess
import tempfile
import threading
import time
import urllib.error
import urllib.request

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--core', type=Path, required=True, help='kinein-core binary')
parser.add_argument('--artifacts', type=Path, default=Path('build/qa-artifacts'))
args = parser.parse_args()
args.artifacts.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='platformio-home-', dir=args.artifacts)).resolve()


class Core:
    """Minimal JSON-RPC client over the core's stdio."""

    def __init__(self, binary, environment):
        self.process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL, text=True, env=environment)
        self.messages, self.sequence = queue.Queue(), 0
        threading.Thread(target=self.read, daemon=True).start()

    def read(self):
        for line in self.process.stdout:
            self.messages.put(json.loads(line))

    def call(self, method, params=None, timeout=180):
        self.sequence += 1
        identity = self.sequence
        self.process.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': identity, 'method': method,
                                             'params': params or {}}) + '\n')
        self.process.stdin.flush()
        end = time.time() + timeout
        while time.time() < end:
            message = self.messages.get(timeout=max(0.01, end - time.time()))
            if message.get('id') == identity and 'method' not in message:
                return message
        raise TimeoutError(method)


def status(url):
    request = urllib.request.Request(url)
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    try:
        with opener.open(request, timeout=5) as response:
            return response.status
    except urllib.error.HTTPError as error:
        return error.code
    except OSError:
        return None


def listening(port):
    with socket.socket() as probe:
        probe.settimeout(1)
        return probe.connect_ex(('127.0.0.1', port)) == 0


# A dead proxy: the core checks its own 127.0.0.1 page and must ignore it.
environment = dict(os.environ, HTTP_PROXY='http://127.0.0.1:9', http_proxy='http://127.0.0.1:9')
environment.pop('NO_PROXY', None)
environment.pop('no_proxy', None)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
report = {'root': str(root), 'core': str(args.core.resolve())}
core = Core(args.core, environment)
try:
    started_at = time.time()
    first = core.call('platformio.homeStart')
    report['startSeconds'] = round(time.time() - started_at, 2)
    assert 'result' in first, first
    url, port = first['result']['url'], first['result']['port']
    report['url'] = url.rsplit('/session/', 1)[0] + '/session/<sessao>/'
    report['sessionLength'] = len(url.rstrip('/').rsplit('/', 1)[1])
    report['sessionStatus'] = status(url)
    report['rootStatus'] = status(f'http://127.0.0.1:{port}/')
    started_at = time.time()
    second = core.call('platformio.homeStart')
    report['reuseSeconds'] = round(time.time() - started_at, 2)
    report['reused'] = second.get('result') == first['result']
    stopped = core.call('platformio.homeStop')
    report['stopped'] = stopped.get('result', {}).get('stopped')
    report['portClosedAfterStop'] = not listening(port)
    again = core.call('platformio.homeStart')
    report['restartedNewSession'] = 'result' in again and again['result']['url'] != url
    last_port = again['result']['port'] if 'result' in again else port
finally:
    core.process.stdin.close()
    try:
        core.process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        core.process.kill()
report['portClosedAfterCoreExit'] = not listening(last_port)
report['passed'] = (report.get('sessionStatus') == 200 and report.get('rootStatus') == 403
                    and report.get('sessionLength') == 32 and report.get('reused') is True
                    and report.get('stopped') is True and report.get('portClosedAfterStop') is True
                    and report.get('restartedNewSession') is True
                    and report.get('portClosedAfterCoreExit') is True)
(root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
print(json.dumps(report, ensure_ascii=False))
assert report['passed'], str(root/'report.json')
