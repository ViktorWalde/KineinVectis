"""Read-only PlatformIO query for the native upload preview (52 §14.44).

The core runs this in PlatformIO's own interpreter (`python -I -c`), in the
project directory and without the removed variables. It reads the computed
configuration, the installed platform and the board through PlatformIO's
API: nothing is installed, SCons does not run and no project script runs.
It prints one JSON object; `error` carries what PlatformIO rejected.
"""
import json
import os

from platformio import __version__
from platformio.platform.factory import PlatformFactory
from platformio.project.config import ProjectConfig


def board_with_overrides(board, options):
    """`board_*` options override the manifest, as the builder's LoadPioPlatform does."""
    for option, value in options.items():
        if not option.startswith("board_"):
            continue
        path = option.lower()[6:]
        try:
            current = board.get(path)
        except KeyError:
            current = None
        if isinstance(current, bool):
            value = str(value).lower() in ("1", "yes", "true")
        elif isinstance(current, int):
            value = int(value)
        board.update(path, value)
    return board


def platform_directory(spec):
    """Where `PlatformFactory.new` loads the platform from, without loading it."""
    if os.path.isdir(spec):
        return spec
    from platformio.package.manager.platform import PlatformPackageManager

    package = PlatformPackageManager().get_package(spec)
    return package.path if package else None


def inside(path, folder):
    path, folder = os.path.realpath(path), os.path.realpath(folder)
    return path == folder or path.startswith(folder + os.sep)


def package_version(directory):
    try:
        with open(os.path.join(directory, "package.json"), encoding="utf8") as handle:
            return json.load(handle).get("version")
    except (OSError, ValueError):
        return None


def query():
    config = ProjectConfig.get_instance()
    config.validate(silent=True)
    envs = config.envs()
    default_envs = config.default_envs()
    candidates = [env for env in envs if not default_envs or env in default_envs]
    result = {
        "core": __version__,
        "configPath": os.path.abspath(config.path),
        "envs": envs,
        "defaultEnvs": default_envs,
        "candidates": candidates,
        "platformio": config.items(section="platformio", as_dict=True),
    }
    if len(candidates) != 1:
        return result
    env = candidates[0]
    options = config.items(env=env, as_dict=True)
    result["env"] = env
    result["options"] = options
    # Loading a platform runs its platform.py: never one the project carries.
    directory = platform_directory(options.get("platform") or "")
    if directory and inside(directory, os.getcwd()):
        return {"error": "a plataforma %s mora dentro do projeto (%s); carregá-la executaria código do projeto"
                % (options.get("platform"), directory)}
    platform = PlatformFactory.from_env(env, autoinstall=False)
    directory = platform.get_dir()
    result["platform"] = {
        "name": platform.name,
        "version": platform.version,
        "files": [
            os.path.join(directory, name)
            for name in ("platform.json", "platform.py", os.path.join("builder", "main.py"))
        ],
    }
    if options.get("board"):
        board = board_with_overrides(platform.board_config(options["board"]), options)
        upload = board.get("upload", {})
        result["board"] = {"id": options["board"], "manifest": board.manifest_path, "upload": upload}
        result["protocol"] = options.get("upload_protocol") or upload.get("protocol")
    uploader = platform.get_package_dir("tool-esptool")
    result["uploader"] = {
        "dir": uploader,
        "version": package_version(uploader) if uploader else None,
    }
    return result


try:
    print(json.dumps(query()))
except Exception as exc:  # pylint: disable=broad-except
    print(json.dumps({"error": "%s: %s" % (type(exc).__name__, exc)}))
