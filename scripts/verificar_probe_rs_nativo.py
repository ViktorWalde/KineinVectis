"""Native probe-rs flash through the real core over stdio (0.4.1h4b2b4b1).

The workspace is a real CMake project built by the core's build.run with
arm-none-eabi; version and chip queries go to the real probe-rs 0.32. Only the
probe listing (when a fixture listing is present) and `download` pass through a
recording wrapper, so no probe is opened and no chip is written. The core runs
with PROBE_RS_* options in its environment; none may reach probe-rs. The real
parser then receives the planned argv with a selector that matches no probe.
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
parser.add_argument('--artifacts', type=Path, default=Path('build/qa-artifacts'))
args = parser.parse_args()
real_probe = shutil.which('probe-rs')
for tool in ('cmake', 'ninja', 'arm-none-eabi-gcc'):
    assert shutil.which(tool), f'{tool} is required for this proof'
assert real_probe, 'probe-rs 0.32 is required for this proof'
args.artifacts.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='probe-rs-native-', dir=args.artifacts)).resolve()
workspace, wrapper_bin, log, home = root/'ws', root/'bin', root/'log', root/'home'
for folder in (workspace, wrapper_bin, log, home):
    folder.mkdir()
PROBE = '0483:374b:066DFF485153'
LISTED = f'The following debug probes were found:\n[0]: STLink V2-1 -- {PROBE} (ST-LINK)\n'
INHERITED = {'PROBE_RS_DRY_RUN': 'true', 'PROBE_RS_ALLOW_ERASE_ALL': 'true',
             'PROBE_RS_CYCLE_POWER': 'true', 'PROBE_RS_REMOTE_HOST': 'example.invalid',
             'PROBE_RS_CHIP_DESCRIPTION_PATH': str(root/'missing.yaml')}

(workspace/'CMakeLists.txt').write_text(
    'cmake_minimum_required(VERSION 3.20)\n'
    'set(CMAKE_SYSTEM_NAME Generic)\nset(CMAKE_SYSTEM_PROCESSOR arm)\n'
    'set(CMAKE_C_COMPILER arm-none-eabi-gcc)\n'
    'set(CMAKE_TRY_COMPILE_TARGET_TYPE STATIC_LIBRARY)\n'
    'project(native_probe C)\n'
    'add_compile_options(-mcpu=cortex-m4 -mthumb -O1)\n'
    'add_library(support STATIC support.c)\n'
    'add_executable(firmware main.c)\n'
    'set_target_properties(firmware PROPERTIES SUFFIX .elf)\n'
    'target_link_options(firmware PRIVATE -mcpu=cortex-m4 -mthumb -nostdlib '
    '-T${CMAKE_CURRENT_SOURCE_DIR}/link.ld)\n')
(workspace/'main.c').write_text(
    'extern unsigned long _estack;\nvoid Reset_Handler(void) { for (;;) { } }\n'
    '__attribute__((section(".isr_vector"), used)) const void *vectors[2] = '
    '{ &_estack, (void *)Reset_Handler };\n')
(workspace/'support.c').write_text('int support(void) { return 1; }\n')
(workspace/'link.ld').write_text(
    'MEMORY { FLASH (rx) : ORIGIN = 0x08000000, LENGTH = 512K '
    'RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 128K }\n'
    '_estack = ORIGIN(RAM) + LENGTH(RAM);\n'
    'SECTIONS { .isr_vector : { KEEP(*(.isr_vector)) } > FLASH '
    '.text : { *(.text*) } > FLASH }\n')
wrapper = wrapper_bin/'probe-rs'
wrapper.write_text(f'''#!/bin/sh
L='{log}'
env | grep '^PROBE_RS_' > "$L/env-$1" || true
pwd > "$L/cwd-$1"
case "$1" in
  list) if [ -f "$L/listing" ]; then cat "$L/listing"; exit 0; fi;;
  download) printf '%s\\n' "$@" > "$L/download.args"; echo wrapper-download-recorded; exit 0;;
  dap-server) exec python3 "$L/fake_dap.py" "$L";;
esac
exec '{real_probe}' "$@"
''')
wrapper.chmod(0o755)
# Minimal stdio DAP stand-in: records the launch; ends when dap.exit appears.
(log/'fake_dap.py').write_text('''import json, os, sys, threading, time
log = sys.argv[1]
seq = 0
def send(message):
    global seq
    seq += 1
    message["seq"] = seq
    data = json.dumps(message).encode()
    sys.stdout.buffer.write(b"Content-Length: %d\\r\\n\\r\\n" % len(data) + data)
    sys.stdout.buffer.flush()
def watch():
    while not os.path.exists(os.path.join(log, "dap.exit")):
        time.sleep(0.02)
    send({"type": "event", "event": "terminated"})
    os._exit(0)
threading.Thread(target=watch, daemon=True).start()
def read():
    headers = {}
    while True:
        line = sys.stdin.buffer.readline()
        if not line:
            return None
        line = line.decode().strip()
        if not line:
            break
        key, value = line.split(":", 1)
        headers[key.strip().lower()] = value.strip()
    return json.loads(sys.stdin.buffer.read(int(headers["content-length"])))
while True:
    request = read()
    if request is None:
        break
    if request.get("command") == "launch":
        with open(os.path.join(log, "dap.launch"), "w") as out:
            json.dump(request.get("arguments"), out)
    send({"type": "response", "request_seq": request["seq"], "command": request.get("command"), "success": True, "body": {}})
    if request.get("command") == "initialize":
        send({"type": "event", "event": "initialized"})
''')


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

    def event(self, predicate, timeout=240):
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


environment = dict(os.environ, HOME=str(home), PATH=f'{wrapper_bin}:{os.environ["PATH"]}', **INHERITED)
for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
    environment[key] = str(root/key.lower())
core = Core(args.core, environment)
report = {'root': str(root), 'core': str(args.core.resolve()), 'coreSha256': digest(args.core),
          'probeRs': real_probe, 'probeRsSha256': digest(real_probe), 'inherited': sorted(INHERITED),
          'version': subprocess.run([real_probe, '--version'], capture_output=True, text=True).stdout.strip()}
try:
    assert 'error' not in core.call('workspace.open', {'path': str(workspace)})

    def job(method, finished_event):
        started = core.call(method, {})
        identity = started['result']['jobId']
        return core.event(lambda m: m.get('method') == finished_event
                          and m['params'].get('jobId') == identity)['params'].get('success')

    report['buildWithoutConfigure'] = job('build.run', 'event.build.finished')
    for method, params in (('toolchain.set', {'role': 'debugAdapter', 'id': 'probe-rs'}),
                           ('toolchain.setKit', {'chip': 'STM32F411RETx'})):
        response = core.call(method, params)
        assert 'error' not in response, response
    request = {'native': True, 'engine': 'probe-rs', 'target': 'firmware', 'probe': PROBE}
    without_reply = core.call('runConfig.flashProposal', request)
    report['withoutFileApiRefusal'] = without_reply.get('error', {}).get('message')
    assert 'file-api' in (report['withoutFileApiRefusal'] or ''), without_reply
    report['configure'] = job('cmake.configure', 'event.cmake.finished')
    report['build'] = job('build.run', 'event.build.finished')
    assert report['configure'] and report['build'], report
    reply = workspace/'.kinein/build/.cmake/api/v1/reply'
    report['replyFiles'] = sorted(path.name for path in reply.iterdir())
    absent = core.call('runConfig.flashProposal', request)
    report['realListingRefusal'] = absent.get('error', {}).get('message')
    assert report['realListingRefusal'] and 'não está na listagem' in report['realListingRefusal'], absent
    library = core.call('runConfig.flashProposal', {**request, 'target': 'support'})
    report['libraryRefusal'] = library.get('error', {}).get('message')
    assert 'não um executável' in (report['libraryRefusal'] or ''), library
    (log/'listing').write_text(LISTED)
    (workspace/'.probe-rs.toml').write_text('[presets.default]\nchip-erase = true\n')
    preset = core.call('runConfig.flashProposal', request)
    report['presetRefusal'] = preset.get('error', {}).get('message')
    assert 'presets' in (report['presetRefusal'] or ''), preset
    (workspace/'.probe-rs.toml').unlink()
    proposal = core.call('runConfig.flashProposal', request)
    assert 'result' in proposal, proposal
    intent = proposal['result']['flash']
    report['proposal'] = {key: proposal['result'][key] for key in ('command', 'source', 'warnings')}
    started = core.call('run.start', {'flash': intent})
    assert 'result' in started, started
    terminal = started['result']['terminalId']
    core.event(lambda m: m.get('method') == 'event.terminal.closed' and m['params'].get('id') == terminal)
    launched = (log/'download.args').read_text().splitlines()
    report['launchedEqualsIntent'] = launched == intent['args']
    report['probeRsEnvironment'] = {path.name: path.read_text() for path in sorted(log.glob('env-*'))}
    report['noInheritedOption'] = all(not text.strip() for text in report['probeRsEnvironment'].values())
    report['downloadWorkingDirectory'] = (log/'cwd-download').read_text().strip()
    artifact = Path(intent['args'][-1])
    report['artifact'] = {'path': str(artifact), 'sha256': digest(artifact),
                          'machine': subprocess.run(['arm-none-eabi-readelf', '-h', str(artifact)],
                                                    capture_output=True, text=True).stdout.count('ARM')}
    debug = core.call('debug.start', {'program': intent['args'][-1], 'probe': PROBE})
    assert 'error' not in debug, debug
    launch = json.loads((log/'dap.launch').read_text())
    busy = core.call('run.start', {'flash': intent})
    (log/'dap.exit').write_text('')
    released = None
    for _ in range(80):
        released = core.call('run.start', {'flash': intent})
        if released.get('error', {}).get('code') != 'SERIAL_PORT_BUSY':
            break
        time.sleep(0.05)
    report['debug'] = {'launchProbe': launch.get('probe'), 'launchChip': launch.get('chip'),
                       'dapServerEnvironment': (log/'env-dap-server').read_text(),
                       'busyWhileDebugging': busy.get('error', {}).get('message'),
                       'releasedAfterAdapterExit': 'result' in released}
    unknown = list(intent['args'])
    unknown[unknown.index('--probe') + 1] = 'ffff:ffff:KINEIN-NO-SUCH-PROBE'
    clean = {key: value for key, value in environment.items() if not key.startswith('PROBE_RS_')}
    parsed = subprocess.run([real_probe, *unknown], cwd=workspace, env=clean, capture_output=True,
                            text=True, timeout=60)
    output = parsed.stdout + parsed.stderr
    report['realParser'] = {'args': unknown, 'exitCode': parsed.returncode,
                            'noProbeSelected': 'No connected probes were found' in output,
                            'usageError': 'Usage:' in output or 'unexpected argument' in output,
                            'tail': output.strip().splitlines()[-1:]}
    report['passed'] = (report['launchedEqualsIntent'] and report['noInheritedOption']
                        and report['debug']['launchProbe'] == PROBE
                        and report['debug']['dapServerEnvironment'] == ''
                        and 'Depuração' in (report['debug']['busyWhileDebugging'] or '')
                        and report['debug']['releasedAfterAdapterExit']
                        and report['downloadWorkingDirectory'] == str(workspace)
                        and report['realParser']['exitCode'] == 1
                        and report['realParser']['noProbeSelected']
                        and not report['realParser']['usageError'])
finally:
    core.process.kill()
    (root/'report.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
print(json.dumps({'report': str(root/'report.json'), 'passed': report.get('passed')}))
assert report.get('passed'), str(root/'report.json')
