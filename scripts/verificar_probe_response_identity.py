"""Exercise the real Qt CoreClient against deliberately reordered probe replies."""
import argparse
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--build-dir', type=Path, required=True)
parser.add_argument('--fixtures', type=Path, default=Path(__file__).resolve().parent/'fixtures/probe-response-identity')
parser.add_argument('--expect-failure', action='store_true')
args = parser.parse_args()
build = args.build_dir.resolve()
fixtures = args.fixtures.resolve()
artifact_base = build.parent/'qa-artifacts'
artifact_base.mkdir(parents=True, exist_ok=True)
root = Path(tempfile.mkdtemp(prefix='probe-response-qt-', dir=artifact_base))
(root/'tmp').mkdir()
task_env = dict(os.environ, TMPDIR=str(root/'tmp'))
flags = shlex.split(subprocess.check_output(['pkg-config', '--cflags', 'Qt6Gui', 'Qt6Qml', 'Qt6Quick'], text=True))
compile_result = subprocess.run(['c++', '-std=c++20', '-fPIC', *flags, '-c', str(fixtures/'main.cpp'), '-o', str(root/'main.o')], capture_output=True, text=True, env=task_env)
(root/'compile.log').write_text(compile_result.stdout + compile_result.stderr)
assert compile_result.returncode == 0, f'helper compilation failed: {root}/compile.log'
commands = subprocess.check_output(['ninja', '-C', str(build), '-t', 'commands', 'ui/kinein-vectis'], text=True)
link = shlex.split(commands.splitlines()[-1].split('&&')[1].strip())
original = [arg for arg in link if arg.endswith('/src/main.cpp.o')]
assert len(original) == 1
link[link.index(original[0])] = str(root/'main.o')
link[link.index('-o')+1] = str(root/'proof')
linked = subprocess.run(link, cwd=build, capture_output=True, text=True, env=task_env)
(root/'link.log').write_text(linked.stdout + linked.stderr)
assert linked.returncode == 0, f'helper link failed: {root}/link.log'
core = root/'core-fixture'
core.write_text('#!' + sys.executable + '\n' + '\n'.join((fixtures/'core.py').read_text().splitlines()[1:]) + '\n')
core.chmod(0o755)
env = task_env.copy()
env.update(QT_QPA_PLATFORM='offscreen', QT_QUICK_BACKEND='software',
           QT_QUICK_CONTROLS_STYLE='Basic', QT_FORCE_STDERR_LOGGING='1',
           KINEIN_CORE_BIN=str(core), KINEIN_PROOF_QML=str(fixtures/'Proof.qml'),
           KINEIN_PROOF_WORKSPACE=str(root/'workspace'))
for key, name in [('XDG_CONFIG_HOME', 'config'), ('XDG_DATA_HOME', 'data'),
                  ('XDG_CACHE_HOME', 'cache'), ('XDG_STATE_HOME', 'state'),
                  ('XDG_RUNTIME_DIR', 'runtime')]:
    area = root/name
    area.mkdir(mode=0o700)
    env[key] = str(area)
(root/'workspace').mkdir()
env.pop('KINEIN_STARTUP_COMMANDS', None)
result = subprocess.run([str(root/'proof')], env=env, cwd=root, capture_output=True, text=True, timeout=15)
output = result.stdout + result.stderr
(root/'run.log').write_text(output)
match = re.search(r'PROBE_RESPONSE_REPORT (\{[^\n]+\})', output)
failed_assertion = result.returncode == 1 and 'PROBE_RESPONSE_FAIL ' in output
report = {'exitCode': result.returncode, 'runtimeAssertionFailed': failed_assertion,
          'passed': result.returncode == 0 and match is not None,
          'result': json.loads(match[1]) if match else None, 'log': str(root/'run.log'),
          'buildDirectory': str(build), 'expectFailure': args.expect_failure}
(root/'report.json').write_text(json.dumps(report, indent=2)+'\n')
print(json.dumps({'report': str(root/'report.json'), **report}))
assert failed_assertion if args.expect_failure else report['passed'], str(root/'run.log')
