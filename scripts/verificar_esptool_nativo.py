"""Native esptool flash through the real core over stdio (0.4.1h4b2b4d1).

The workspace is an ESP-IDF build fixture (recipe, project description and
image); the core plans from it as from a real build. esptool is a recording
wrapper in PATH: `version` goes to the real esptool, the flash call is only
recorded, and the port is a regular file, so no device is opened. The core
runs with ESPTOOL_* options in its environment, ESPTOOL_CFGFILE included;
none may reach esptool. An `[esptool]` file in the root or in the user
configuration folder refuses the preview, and one created after saving
refuses run.start (52 §14.38).
"""
import argparse
import hashlib
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
parser.add_argument('--esptool', type=Path, default=shutil.which('esptool'), help='real esptool 4.8 or 5.x')
parser.add_argument('--artifacts', type=Path, default=Path('build/qa-artifacts'))
args = parser.parse_args()
assert args.esptool and Path(args.esptool).is_file(), 'a real esptool is required for this proof'
args.artifacts.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='esptool-native-', dir=args.artifacts)).resolve()
workspace, wrapper_bin, log, home = root/'ws', root/'bin', root/'log', root/'home'
for folder in (workspace, wrapper_bin, log, home, workspace/'build'):
    folder.mkdir()
injected = root/'injected.cfg'
injected.write_text('[esptool]\nconnect_attempts = 1\n')
INHERITED = {'ESPTOOL_CFGFILE': str(injected), 'ESPTOOL_STUB_VERSION': '1',
             'ESPTOOL_CONNECT_ATTEMPTS': '1', 'ESPTOOL_BEFORE': 'no-reset'}
port = root/'port'
port.write_text('regular fixture; esptool never opens it\n')

(workspace/'CMakeLists.txt').write_text(
    'include($ENV{IDF_PATH}/tools/cmake/project.cmake)\nproject(native)\n')
(workspace/'sdkconfig').write_text('CONFIG_IDF_TARGET="esp32"\n')
(workspace/'build/app.bin').write_bytes(b'compiled fixture image\n')
(workspace/'build/flasher_args.json').write_text(json.dumps({
    'flash_files': {'0x10000': 'app.bin'},
    'flash_settings': {'flash_mode': 'dio', 'flash_size': '4MB', 'flash_freq': '40m'},
    'extra_esptool_args': {'chip': 'esp32', 'stub': False, 'before': 'default-reset',
                           'after': 'hard-reset'}}))
(workspace/'build/config').mkdir()
(workspace/'build/config/sdkconfig.json').write_text(json.dumps({
    'IDF_TARGET': 'esp32', 'SECURE_BOOT': False, 'SECURE_FLASH_ENC_ENABLED': False}))
(workspace/'build/project_description.json').write_text(json.dumps({
    'version': '1.3', 'project_path': str(workspace), 'build_dir': str(workspace/'build'),
    'config_file': str(workspace/'sdkconfig'), 'target': 'esp32', 'app_bin': 'app.bin'}))
wrapper = wrapper_bin/'esptool'
wrapper.write_text(f'''#!/bin/sh
L='{log}'
call=flash
if [ "$1" = version ]; then call=version; fi
env | grep '^ESPTOOL_' > "$L/env-$call" || true
pwd > "$L/cwd-$call"
if [ "$call" = version ]; then exec '{args.esptool}' version; fi
printf '%s\\n' "$@" > "$L/flash.args"
echo wrapper-flash-recorded
''')
wrapper.chmod(0o755)


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

    def call(self, method, params=None, timeout=60):
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

    def event(self, predicate, timeout=60):
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


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def refusal(response):
    return response.get('error', {}).get('message') or ''


environment = dict(os.environ, HOME=str(home), PATH=f'{wrapper_bin}:{os.environ["PATH"]}', **INHERITED)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
for key in ('IDF_PATH', 'ESPPORT', 'ESPBAUD'):
    environment.pop(key, None)
core = Core(args.core, environment)
report = {'root': str(root), 'core': str(args.core.resolve()), 'coreSha256': digest(args.core),
          'esptool': str(args.esptool), 'inherited': sorted(INHERITED),
          'version': subprocess.run([str(args.esptool), 'version'], capture_output=True, text=True,
                                    env={k: v for k, v in environment.items()
                                         if not k.startswith('ESPTOOL_')}).stdout.strip()}
request = {'native': True, 'device': str(port)}
try:
    assert 'error' not in core.call('workspace.open', {'path': str(workspace)})
    (workspace/'esptool.cfg').write_text('[esptool]\ncustom_reset_sequence = D0|R1|W0.1\n')
    report['rootConfigRefusal'] = refusal(core.call('runConfig.flashProposal', request))
    report['noProcessBeforeRefusal'] = not (log/'env-version').exists()
    (workspace/'esptool.cfg').unlink()
    user = home/'.config/esptool'
    user.mkdir(parents=True)
    (user/'setup.cfg').write_text('[flake8]\n\n[esptool]\nreset_delay = 2\n')
    report['userConfigRefusal'] = refusal(core.call('runConfig.flashProposal', request))
    (user/'setup.cfg').unlink()
    proposal = core.call('runConfig.flashProposal', request)
    assert 'result' in proposal, proposal
    intent = proposal['result']['flash']
    report['proposal'] = {key: proposal['result'].get(key) for key in ('command', 'source', 'warnings')}
    saved = core.call('runConfig.save', {'name': 'Gravar (esptool)', 'command': proposal['result']['command'],
                                         'flash': intent})
    assert 'error' not in saved, saved
    (workspace/'tox.ini').write_text('[esptool]\nconnect_attempts = 1\n')
    report['afterSaveRefusal'] = refusal(core.call('run.start', {}))
    report['notLaunchedAfterSave'] = not (log/'flash.args').exists()
    (workspace/'tox.ini').unlink()
    started = core.call('run.start', {})
    assert 'result' in started, started
    terminal = started['result']['terminalId']
    core.event(lambda m: m.get('method') == 'event.terminal.closed' and m['params'].get('id') == terminal)
    launched = (log/'flash.args').read_text().splitlines()
    report['launchedEqualsIntent'] = launched == intent['args']
    report['esptoolEnvironment'] = {path.name: path.read_text() for path in sorted(log.glob('env-*'))}
    report['noInheritedOption'] = (set(report['esptoolEnvironment']) == {'env-version', 'env-flash'}
                                   and all(not text.strip() for text in report['esptoolEnvironment'].values()))
    report['flashWorkingDirectory'] = (log/'cwd-flash').read_text().strip()
    source = ' '.join(report['proposal']['source'])
    report['passed'] = ('esptool.cfg' in report['rootConfigRefusal']
                        and '[esptool]' in report['rootConfigRefusal']
                        and report['noProcessBeforeRefusal']
                        and 'setup.cfg' in report['userConfigRefusal']
                        and 'tox.ini' in report['afterSaveRefusal']
                        and report['notLaunchedAfterSave']
                        and report['launchedEqualsIntent']
                        and report['noInheritedOption']
                        and all(name in source for name in INHERITED)
                        and report['flashWorkingDirectory'] == str(workspace))
finally:
    core.process.kill()
    (root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
print(json.dumps({'report': str(root/'report.json'), 'passed': report.get('passed')}))
assert report.get('passed'), str(root/'report.json')
