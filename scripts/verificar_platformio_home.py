"""PIO Home through the real core and the real PlatformIO (0.4.1h4b2b4h, h4b2b4i).

Needs `pio` (PlatformIO Core 6) in PATH. The core starts PIO Home with
`platformio.homeStart`: the session page must answer 200 and the server root
403, a second start must reuse the same session, `platformio.homeStop` must
close the port, and after the core exits nothing may still listen. Nothing is
installed beyond what `pio home` installs itself on its first run. The core
runs with a dead HTTP proxy in its environment: its readiness check is local
and must not go through it (52 §14.45).

The IDE agent (52 §14.46): a client in the page's role sends `ide.send_command`
to the real server, and the core's agent must answer it (the open workspace as
the project dirs, `true` and the UI event for an absolute path, `false` and no
event for a relative one). After the core exits, and after it is killed, the
agent process must be gone. `platformio.homeRelease`: a start cancels it, and
its deadline stops the server with `event.platformio.homeExited` "idle".
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import queue
import signal
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
# The agent's own WebSocket client plays the page (the server is the real one).
spec = importlib.util.spec_from_file_location(
    'agent', Path(__file__).resolve().parent.parent/'crates/kinein-core/src/platformio/agent.py')
agent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(agent)


class Core:
    """Minimal JSON-RPC client over the core's stdio."""

    def __init__(self, binary, environment):
        self.process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL, text=True, env=environment)
        self.messages, self.sequence, self.events = queue.Queue(), 0, queue.Queue()
        threading.Thread(target=self.read, daemon=True).start()

    def read(self):
        for line in self.process.stdout:
            message = json.loads(line)
            (self.events if 'id' not in message else self.messages).put(message)

    def first_event(self, method, wait):
        """The params of the next `method` event and when it arrived, or None."""
        end = time.time() + wait
        while time.time() < end:
            try:
                event = self.events.get(timeout=max(0.01, end - time.time()))
            except queue.Empty:
                break
            if event.get('method') == method:
                return event['params'], time.time()
        return None

    def events_named(self, method, wait):
        found, end = [], time.time() + wait
        while time.time() < end:
            try:
                event = self.events.get(timeout=max(0.01, end - time.time()))
            except queue.Empty:
                break
            if event.get('method') == method:
                found.append(event['params'])
        return found

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


def page_commands(url, commands):
    """Send each command as the page does and return the server's answers."""
    page = agent.Socket(url.replace('http://', 'ws://', 1) + 'wsrpc')
    answers = []
    for identity, command in enumerate(commands, 1):
        page.send(0x1, json.dumps({'jsonrpc': '2.0', 'id': identity, 'method': 'ide.send_command',
                                   'params': command}).encode())
        while True:
            reply = json.loads(page.message())
            if reply.get('id') == identity:
                answers.append(reply.get('result', reply.get('error')))
                break
    page.send(0x8, b'\x03\xe8')
    return answers


def agents_of(url):
    """The agent processes still running for this session: its argument is the
    WebSocket URL, which the server's own command line does not have."""
    session = url.rstrip('/').rsplit('/', 1)[1]
    found = subprocess.run(['pgrep', '-f', f'/session/{session}/wsrpc'], capture_output=True,
                           text=True).stdout.split()
    return [pid for pid in found if pid != str(os.getpid())]


def gone_within(url, seconds):
    end = time.time() + seconds
    while agents_of(url) and time.time() < end:
        time.sleep(0.2)
    return not agents_of(url)


# A dead proxy: the core checks its own 127.0.0.1 page and must ignore it.
environment = dict(os.environ, HTTP_PROXY='http://127.0.0.1:9', http_proxy='http://127.0.0.1:9')
environment.pop('NO_PROXY', None)
environment.pop('no_proxy', None)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
report = {'root': str(root), 'core': str(args.core.resolve())}
project = root/'project'
(project/'src').mkdir(parents=True)
(project/'platformio.ini').write_text('[env:esp32dev]\nplatform = espressif32\nboard = esp32dev\n')
core = Core(args.core, environment)
try:
    opened = core.call('workspace.open', {'path': str(project)})
    assert 'result' in opened, opened
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
    source = str(project/'src/main.cpp')
    answers = page_commands(url, [
        ['get_pio_project_dirs'],
        ['open_project', str(project)],
        ['open_text_document', {'path': source, 'line': 3, 'column': 2}],
        ['open_project', 'relative/dir'],
    ])
    report['agentAnswers'] = [answers[0], answers[1], answers[2], answers[3]]
    events = core.events_named('event.platformio.homeCommand', 3)
    report['agentEvents'] = events
    report['agentPassed'] = (
        answers == [[str(project)], True, True, False]
        and events == [{'method': 'open_project', 'params': {'path': str(project)}},
                       {'method': 'open_text_document',
                        'params': {'path': source, 'line': 3, 'column': 2}}])
    started_at = time.time()
    stopped = core.call('platformio.homeStop')
    report['stopSeconds'] = round(time.time() - started_at, 2)
    report['agentGoneAfterStop'] = gone_within(url, 5)
    report['stopped'] = stopped.get('result', {}).get('stopped')
    report['portClosedAfterStop'] = not listening(port)
    again = core.call('platformio.homeStart')
    report['restartedNewSession'] = 'result' in again and again['result']['url'] != url
    last_port = again['result']['port'] if 'result' in again else port
    last_url = again['result']['url'] if 'result' in again else url

    # The release (52 §14.46): a start before the deadline cancels it; the
    # deadline stops the server as "idle", on the core's clock.
    core.call('platformio.homeRelease', {'after': 2})
    reuse = core.call('platformio.homeStart')
    time.sleep(3)
    report['releaseCancelledByStart'] = (reuse.get('result') == again.get('result')
                                         and status(last_url) == 200)
    released_at = time.time()
    releasing = core.call('platformio.homeRelease', {'after': 1})
    idle = core.first_event('event.platformio.homeExited', 15)
    report['releasing'] = releasing.get('result', {}).get('releasing')
    report['releaseEvent'] = idle[0] if idle else None
    report['releaseSeconds'] = round(idle[1] - released_at, 2) if idle else None
    report['portClosedAfterRelease'] = not listening(last_port)
    report['agentGoneAfterRelease'] = gone_within(last_url, 5)
    # One more server, for the checks after the core exits.
    final = core.call('platformio.homeStart')
    assert 'result' in final, final
    last_port, last_url = final['result']['port'], final['result']['url']
finally:
    core.process.stdin.close()
    try:
        core.process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        core.process.kill()
report['portClosedAfterCoreExit'] = not listening(last_port)
report['agentGoneAfterCoreExit'] = gone_within(last_url, 5)

# A core that dies without stopping anything: the agent sees its stdin close
# and leaves; the server, with no client, is left to its own idle shutdown,
# which this proof does not wait for (it shuts it down by hand).
killed = Core(args.core, environment)
orphan = killed.call('platformio.homeStart')
assert 'result' in orphan, orphan
killed.process.send_signal(signal.SIGKILL)
killed.process.wait(timeout=15)
report['agentGoneAfterCoreKill'] = gone_within(orphan['result']['url'], 10)
report['serverLeftToItsIdleShutdown'] = listening(orphan['result']['port'])
shutdown = urllib.request.Request(orphan['result']['url'] + '__shutdown__', method='POST')
try:
    urllib.request.build_opener(urllib.request.ProxyHandler({})).open(shutdown, timeout=5).read()
except OSError:
    pass
report['passed'] = (report.get('sessionStatus') == 200 and report.get('rootStatus') == 403
                    and report.get('sessionLength') == 32 and report.get('reused') is True
                    and report.get('stopped') is True and report.get('portClosedAfterStop') is True
                    and report.get('restartedNewSession') is True
                    and report.get('portClosedAfterCoreExit') is True
                    and report.get('agentPassed') is True
                    and report.get('agentGoneAfterStop') is True
                    and report.get('agentGoneAfterCoreExit') is True
                    and report.get('agentGoneAfterCoreKill') is True
                    and report.get('releaseCancelledByStart') is True
                    and report.get('releasing') is True
                    and report.get('releaseEvent') == {'code': None, 'reason': 'idle'}
                    and report.get('releaseSeconds') is not None
                    and 1 <= report['releaseSeconds'] < 6
                    and report.get('portClosedAfterRelease') is True
                    and report.get('agentGoneAfterRelease') is True)
(root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
print(json.dumps(report, ensure_ascii=False))
assert report['passed'], str(root/'report.json')
