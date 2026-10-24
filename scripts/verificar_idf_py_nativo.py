"""Native idf.py flash through the real core over stdio (0.4.1h4b2b4d3).

The workspace is an ESP-IDF build fixture; HOME holds a fake EIM activation
that puts a recording idf.py first in PATH, behind the real bash. The core
runs with ESPPORT, ESPBAUD, IDF_FLASH_FULL, IDF_TRUST_FLASH_CONTENT,
SERIAL_TOOL_EXTRA_ARGS, ESPTOOL_CFGFILE, IDF_EXTRA_ACTIONS_PATH and
IDF_EXTENSION_ALLOW_UNTRUSTED in its environment; none may reach idf.py,
neither in the version query nor in the flash. The port is a regular
file and nothing is flashed (52 §14.40).
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import tempfile
import threading
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--core', type=Path, required=True, help='kinein-core binary')
parser.add_argument('--artifacts', type=Path, default=Path('build/qa-artifacts'))
args = parser.parse_args()
args.artifacts.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='idf-py-native-', dir=args.artifacts)).resolve()
workspace, idf_bin, log, home = root/'ws', root/'idf-bin', root/'log', root/'home'
for folder in (workspace, idf_bin, log, home/'.espressif/tools', workspace/'build/config'):
    folder.mkdir(parents=True)
INHERITED = {'ESPPORT': '/dev/inherited', 'ESPBAUD': '9600', 'IDF_FLASH_FULL': '1',
             'IDF_TRUST_FLASH_CONTENT': '1', 'SERIAL_TOOL_EXTRA_ARGS': '--force',
             'ESPTOOL_CFGFILE': str(root/'injected.cfg'), 'IDF_EXTRA_ACTIONS_PATH': str(root/'actions'),
             'IDF_EXTENSION_ALLOW_UNTRUSTED': '1'}
(root/'injected.cfg').write_text('[esptool]\nconnect_attempts = 1\n')
port = root/'port'
port.write_text('regular fixture; idf.py never opens it\n')
(workspace/'CMakeLists.txt').write_text(
    'include($ENV{IDF_PATH}/tools/cmake/project.cmake)\nproject(native)\n')
(workspace/'sdkconfig').write_text('CONFIG_IDF_TARGET="esp32"\n')
(workspace/'build/app.bin').write_bytes(b'compiled fixture image\n')
(workspace/'build/flasher_args.json').write_text(json.dumps({
    'flash_files': {'0x10000': 'app.bin'},
    'extra_esptool_args': {'chip': 'esp32', 'stub': False, 'before': 'default-reset',
                           'after': 'hard-reset'}}))
(workspace/'build/config/sdkconfig.json').write_text(json.dumps({
    'IDF_TARGET': 'esp32', 'SECURE_BOOT': False, 'SECURE_FLASH_ENC_ENABLED': False}))
(workspace/'build/project_description.json').write_text(json.dumps({
    'version': '1.3', 'project_path': str(workspace), 'build_dir': str(workspace/'build'),
    'config_file': str(workspace/'sdkconfig'), 'target': 'esp32', 'app_bin': 'app.bin'}))
(home/'.espressif/tools/activate_idf_v6.1.sh').write_text(f"export PATH='{idf_bin}':\"$PATH\"\n")
watched = "'^(ESPTOOL_|ESPPORT=|ESPBAUD=|IDF_FLASH_FULL=|IDF_TRUST_FLASH_CONTENT=|SERIAL_TOOL_EXTRA|IDF_EXTRA_ACTIONS_PATH=|IDF_EXTENSION_ALLOW)'"
fake = idf_bin/'idf.py'
fake.write_text(f"""#!/bin/sh
L='{log}'
if [ "$1" = --version ]; then env | grep -E {watched} > "$L/env-version"; echo 'ESP-IDF v6.1'; exit 0; fi
env | grep -E {watched} > "$L/env-flash"
pwd > "$L/cwd-flash"
printf '%s\\n' "$@" > "$L/flash.args"
echo idf-fixture-complete
""")
fake.chmod(0o755)


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


environment = dict(os.environ, HOME=str(home), **INHERITED)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
environment.pop('IDF_PATH', None)
core = Core(args.core, environment)
report = {'root': str(root), 'core': str(args.core.resolve()), 'coreSha256': digest(args.core),
          'inherited': sorted(INHERITED)}
try:
    assert 'error' not in core.call('workspace.open', {'path': str(workspace)})
    proposal = core.call('runConfig.flashProposal', {'native': True, 'engine': 'idf.py', 'device': str(port)})
    assert 'result' in proposal, proposal
    intent = proposal['result']['flash']
    report['proposal'] = {key: proposal['result'].get(key) for key in ('command', 'source', 'warnings')}
    started = core.call('run.start', {'flash': intent})
    assert 'result' in started, started
    terminal = started['result']['terminalId']
    core.event(lambda m: m.get('method') == 'event.terminal.closed' and m['params'].get('id') == terminal)
    launched = (log/'flash.args').read_text().splitlines()
    report['launchedTail'] = launched
    report['intentTail'] = intent['args'][-7:]
    report['idfEnvironment'] = {path.name: path.read_text() for path in sorted(log.glob('env-*'))}
    report['noInheritedOption'] = (set(report['idfEnvironment']) == {'env-version', 'env-flash'}
                                   and all(not text.strip() for text in report['idfEnvironment'].values()))
    report['flashWorkingDirectory'] = (log/'cwd-flash').read_text().strip()
    source = ' '.join(report['proposal']['source'])
    report['passed'] = (launched == intent['args'][-7:]
                        and report['noInheritedOption']
                        and all(name in source for name in INHERITED)
                        and report['flashWorkingDirectory'] == str(workspace))
finally:
    core.process.kill()
    (root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
print(json.dumps({'report': str(root/'report.json'), 'passed': report.get('passed')}))
assert report.get('passed'), str(root/'report.json')
