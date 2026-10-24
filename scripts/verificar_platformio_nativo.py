"""Native PlatformIO upload through the real core and the real PlatformIO (0.4.1h4b2b4d4).

Needs `pio` (PlatformIO Core 6) in PATH with the espressif32 platform and the
Arduino framework installed. Each fixture project goes through
runConfig.flashProposal with the real `pio system info` and the real
configuration query; the ones the preview cannot promise must be refused.
The core runs with PLATFORMIO_UPLOAD_FLAGS, PLATFORMIO_BUILD_FLAGS,
PLATFORMIO_DEFAULT_ENVS and ESPTOOL_BAUD set: a leak would change or break the
answers. One accepted project then runs through run.start: the real `pio run`
builds it and the real esptool fails on the port, a regular file. No board is
touched and nothing is flashed (52 §14.44).
"""
import argparse
import json
import os
from pathlib import Path
import queue
import shutil
import subprocess
import tempfile
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--core', type=Path, required=True, help='kinein-core binary')
parser.add_argument('--artifacts', type=Path, default=Path('build/qa-artifacts'))
args = parser.parse_args()
args.artifacts.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='platformio-native-', dir=args.artifacts)).resolve()
port = root/'port'
port.write_text('regular fixture; esptool cannot configure it\n')
SOURCE = '#include <Arduino.h>\nvoid setup() {}\nvoid loop() {}\n'
ESP32 = '[env:esp32dev]\nplatform = espressif32\nboard = esp32dev\nframework = arduino\n'
# name -> (platformio.ini, expected: None = accepted, or a text the refusal names)
FIXTURES = {
    'bench': (ESP32, None),
    'default-env': ('[platformio]\ndefault_envs = esp32dev\n\n' + ESP32
                    + '\n[env:other]\nplatform = espressif32\nboard = esp32dev\nframework = arduino\n',
                    None),
    'ini-port': (ESP32 + 'upload_port = 192.168.0.10\n', None),
    'sysenv-speed': (ESP32 + 'upload_speed = ${sysenv.KINEIN_PIO_SPEED}\n', None),
    'two-envs': (ESP32 + '\n[env:other]\nplatform = espressif32\nboard = esp32dev\nframework = arduino\n',
                 'default_envs'),
    'scripts': (ESP32 + 'extra_scripts = pre:version.py\n', 'extra_scripts'),
    'command': (ESP32 + 'upload_command = cp $SOURCE /tmp/x\n', 'upload_command'),
    'packages': (ESP32 + 'platform_packages = platformio/tool-esptool\n', 'platform_packages'),
    'configs': ('[platformio]\nextra_configs = extra.ini\n\n' + ESP32, 'extra_configs'),
    'ota': (ESP32 + 'upload_protocol = espota\n', 'espota'),
    'jtag': (ESP32 + 'upload_protocol = esp-prog\n', 'esp-prog'),
    'touch-board': ('[env:s3]\nplatform = espressif32\nboard = aventen_s3_sync\nframework = arduino\n',
                    'use_1200bps_touch'),
    'touch-override': (ESP32 + 'board_upload.wait_for_upload_port = yes\n', 'wait_for_upload_port'),
    'espidf': ('[env:esp32dev]\nplatform = espressif32\nboard = esp32dev\nframework = espidf\n', 'espidf'),
    'avr': ('[env:uno]\nplatform = atmelavr\nboard = uno\nframework = arduino\n', 'recusou a configuração'),
    'unknown-default': ('[platformio]\ndefault_envs = nope\n\n' + ESP32, 'recusou a configuração'),
    # A platform the project carries: loading it would run its platform.py.
    'local-platform': ('[env:x]\nplatform = ./myplat\nboard = esp32dev\nframework = arduino\n',
                       'mora dentro do projeto'),
}
INHERITED = {'PLATFORMIO_UPLOAD_FLAGS': '--kinein-leak', 'PLATFORMIO_BUILD_FLAGS': '-DKINEIN_LEAK=1',
             'PLATFORMIO_DEFAULT_ENVS': 'kinein-leak', 'ESPTOOL_BAUD': '9600'}


class Core:
    """Minimal JSON-RPC client over the core's stdio."""

    def __init__(self, binary, environment):
        self.process = subprocess.Popen([str(binary)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                        stderr=subprocess.DEVNULL, text=True, env=environment)
        self.messages, self.pending, self.sequence = queue.Queue(), [], 0
        threading.Thread(target=self.read, daemon=True).start()

    def read(self):
        for line in self.process.stdout:
            self.messages.put(json.loads(line))

    def call(self, method, params=None, timeout=120):
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
            self.pending.append(message)
        raise TimeoutError(method)

    def event(self, predicate, timeout):
        for message in list(self.pending):
            if predicate(message):
                self.pending.remove(message)
                return message
        end = time.time() + timeout
        while time.time() < end:
            message = self.messages.get(timeout=max(0.01, end - time.time()))
            if predicate(message):
                return message
            self.pending.append(message)
        raise TimeoutError('event')


environment = dict(os.environ, KINEIN_PIO_SPEED='115200', **INHERITED)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
core = Core(args.core, environment)
report = {'root': str(root), 'core': str(args.core.resolve()), 'inherited': sorted(INHERITED),
          'pio': shutil.which('pio'), 'fixtures': {}}
try:
    for name, (ini, expected) in FIXTURES.items():
        project = root/name
        (project/'src').mkdir(parents=True)
        (project/'platformio.ini').write_text(ini)
        (project/'src/main.cpp').write_text(SOURCE)
        if name == 'local-platform':
            (project/'myplat').mkdir()
            (project/'myplat/platform.json').write_text(json.dumps({'name': 'espressif32', 'version': '0.0.0'}))
            (project/'myplat/platform.py').write_text(
                f"open({str(root/'local-platform-ran')!r}, 'w').write('ran')\n")
        assert 'error' not in core.call('workspace.open', {'path': str(project)})
        started = time.time()
        answer = core.call('runConfig.flashProposal',
                           {'native': True, 'engine': 'platformio', 'device': str(port)})
        entry = {'expected': expected, 'seconds': round(time.time() - started, 2)}
        if 'result' in answer:
            entry['args'] = answer['result']['flash']['args']
            entry['source'] = answer['result']['source']
            entry['intent'] = answer['result']['flash']
            entry['passed'] = expected is None
        else:
            entry['refusal'] = answer['error']['message']
            entry['passed'] = expected is not None and expected in entry['refusal']
        report['fixtures'][name] = entry
    bench = report['fixtures']['bench']
    source = ' '.join(bench.get('source', []))
    report['sourceNamesRemoved'] = all(name in source for name in INHERITED)
    report['defaultEnvPinned'] = '-e' in report['fixtures']['default-env'].get('args', []) and \
        report['fixtures']['default-env']['args'][report['fixtures']['default-env']['args'].index('-e') + 1] == 'esp32dev'
    report['sysenvSpeed'] = any('115200 (upload_speed' in line
                                for line in report['fixtures']['sysenv-speed'].get('source', []))
    # The bench intent runs: the real pio builds, the real esptool meets a regular file.
    assert 'error' not in core.call('workspace.open', {'path': str(root/'bench')})
    started = core.call('run.start', {'flash': bench['intent']})
    report['runStarted'] = 'result' in started
    if report['runStarted']:
        terminal = started['result']['terminalId']
        closed = core.event(lambda m: m.get('method') == 'event.terminal.closed'
                            and m['params'].get('id') == terminal, timeout=600)
        report['runExit'] = closed['params'].get('exitCode')
        selection = {'id': terminal, 'selectionId': 'platformio-proof'}
        core.call('terminal.selectAll', selection)
        text = core.call('terminal.copySelection', selection)['result']['text']
        (root/'run.txt').write_text(text)
        report['runReachedEsptool'] = 'Could not open' in text and str(port) in text
        report['runLeaked'] = any(marker in text for marker in ('kinein-leak', 'KINEIN_LEAK', ' 9600'))
    report['localPlatformNotRun'] = not (root/'local-platform-ran').exists()
    report['portUntouched'] = port.read_text() == 'regular fixture; esptool cannot configure it\n'
    report['passed'] = (all(entry['passed'] for entry in report['fixtures'].values())
                        and report['sourceNamesRemoved'] and report['defaultEnvPinned']
                        and report['sysenvSpeed'] and report.get('runReachedEsptool') is True
                        and report.get('runLeaked') is False and report['runExit'] not in (0, None)
                        and report['portUntouched'] and report['localPlatformNotRun'])
finally:
    core.process.kill()
    (root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
failed = [name for name, entry in report['fixtures'].items() if not entry.get('passed')]
print(json.dumps({'report': str(root/'report.json'), 'passed': report.get('passed'), 'failed': failed}))
assert report.get('passed'), str(root/'report.json')
