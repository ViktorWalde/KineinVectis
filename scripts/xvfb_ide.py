"""A IDE num Xvfb, dirigida por mouse e teclado (`xdotool`), com fotos a
qualquer momento (Pillow `ImageGrab` pelo XCB).

Para os aceites com mouse e teclado que o modo autônomo delegou ao agente
(roadmaps/40 §5, "modo autonomo"; 59 §7.1, fatia 16.1): o gesto é real (o X
entrega o evento à janela, como faria um mouse), mas quem o faz é o agente,
e o registro diz isso. HOME e XDG isolados num diretório temporário: nada da
pessoa é lido nem gravado.

    from xvfb_ide import Ide
    with Ide(projeto, fotos) as ide:
        ide.click(110, 150)
        ide.key("ctrl+s")
        ide.shot("depois")
"""
from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parent.parent


def _first_existing(*candidates: Path) -> Path:
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


_MODIFIERS = {"ctrl": "Control_L", "control": "Control_L", "shift": "Shift_L",
              "alt": "Alt_L", "super": "Super_L"}


def _press_combo(display: str, combo: str) -> None:
    """Modificadores na ordem, a tecla, e a soltura ao contrário (XTEST)."""
    import ctypes

    x11 = ctypes.CDLL("libX11.so.6")
    xtst = ctypes.CDLL("libXtst.so.6")
    x11.XOpenDisplay.restype = ctypes.c_void_p
    x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    x11.XStringToKeysym.restype = ctypes.c_ulong
    x11.XStringToKeysym.argtypes = [ctypes.c_char_p]
    x11.XKeysymToKeycode.restype = ctypes.c_ubyte
    x11.XKeysymToKeycode.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    x11.XFlush.argtypes = [ctypes.c_void_p]
    x11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    xtst.XTestFakeKeyEvent.argtypes = [ctypes.c_void_p, ctypes.c_uint, ctypes.c_int,
                                       ctypes.c_ulong]
    connection = x11.XOpenDisplay(display.encode())
    if not connection:
        raise RuntimeError(f"sem conexão com o display {display}")
    try:
        codes = []
        for name in combo.split("+"):
            symbol = x11.XStringToKeysym(_MODIFIERS.get(name.lower(), name).encode())
            code = x11.XKeysymToKeycode(connection, symbol) if symbol else 0
            if not code:
                raise RuntimeError(f"tecla desconhecida no display {display}: {name}")
            codes.append(code)
        for code in codes:
            xtst.XTestFakeKeyEvent(connection, code, 1, 0)
        for code in reversed(codes):
            xtst.XTestFakeKeyEvent(connection, code, 0, 0)
        x11.XFlush(connection)
    finally:
        x11.XCloseDisplay(connection)


class Ide:
    """Uma sessão da IDE num Xvfb próprio."""

    def __init__(self, project: Path, shots: Path, *, size: str = "1366x768",
                 ui: Path | None = None, core: Path | None = None,
                 startup: str = "", env: dict[str, str] | None = None,
                 settle: float = 10.0):
        self.project = Path(project)
        self.shots = Path(shots)
        self.size = size
        self.ui = ui or _first_existing(REPO / "build/dev-local/ui/kinein-vectis",
                                        REPO / "build/dev-local-release/ui/kinein-vectis")
        self.core = core or _first_existing(REPO / "target/debug/kinein-core",
                                            REPO / "target/release/kinein-core")
        self.startup = startup
        self.extra_env = env or {}
        self.settle = settle
        self.base = Path(tempfile.mkdtemp(prefix="kinein-xvfb-"))
        self.display = ""
        self.xvfb: subprocess.Popen | None = None
        self.app: subprocess.Popen | None = None
        self.log = self.base / "ide.log"
        self.count = 0

    def __enter__(self) -> "Ide":
        self.start()
        return self

    def __exit__(self, *_exc) -> None:
        self.stop()

    def start(self) -> None:
        self.shots.mkdir(parents=True, exist_ok=True)
        home = self.base / "home"
        home.mkdir()
        # O `~/.cargo/bin` da pessoa vem na frente: sem ele o Project Health
        # acusa cargo e rust-analyzer ausentes (o HOME aqui e' falso).
        cargo_bin = Path(os.environ.get("CARGO_HOME", str(Path.home() / ".cargo"))) / "bin"
        path = os.environ.get("PATH", "/usr/bin:/bin")
        if cargo_bin.is_dir() and str(cargo_bin) not in path.split(":"):
            path = f"{cargo_bin}:{path}"
        env = {"HOME": str(home), "PATH": path,
               "LANG": "C.UTF-8", "QT_QPA_PLATFORM": "xcb",
               "RUSTUP_HOME": os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")),
               "CARGO_HOME": os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
               "KINEIN_CORE_BIN": str(self.core)}
        for name in ("config", "data", "cache", "state", "runtime"):
            path = self.base / f"xdg-{name}"
            path.mkdir(mode=0o700)
            key = "XDG_RUNTIME_DIR" if name == "runtime" else f"XDG_{name.upper()}_HOME"
            env[key] = str(path)
        if self.startup:
            env["KINEIN_STARTUP_COMMANDS"] = self.startup
        env.update(self.extra_env)
        width, height = self.size.split("x")
        for number in range(90, 100):
            if not Path(f"/tmp/.X11-unix/X{number}").exists():
                self.display = f":{number}"
                break
        self.xvfb = subprocess.Popen(
            ["Xvfb", self.display, "-screen", "0", f"{width}x{height}x24", "-dpi", "96",
             "-nolisten", "tcp"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        time.sleep(1.0)
        env["DISPLAY"] = self.display
        self.env = env
        with self.log.open("w") as log:
            self.app = subprocess.Popen([str(self.ui), str(self.project)], env=env,
                                        stdout=log, stderr=log)
        time.sleep(self.settle)
        if self.app.poll() is not None:
            # Morreu na abertura (um tipo QML que nao carregou, por exemplo):
            # o `--sync` abaixo esperaria para sempre por uma janela.
            tail = self.log.read_text(errors="replace")[-2000:]
            code = self.stop()
            raise RuntimeError(f"a IDE saiu com {code} ao abrir:\n{tail}")
        # A janela ocupa a tela inteira, como o script de fotos faz.
        self.xdo("search", "--sync", "--onlyvisible", "--class", "kinein", "windowsize",
                 "%@", width, height, check=False)
        self.xdo("search", "--onlyvisible", "--class", "kinein", "windowmove", "%@", "0", "0",
                 check=False)
        time.sleep(1.0)

    def xdo(self, *args: str, check: bool = True) -> str:
        result = subprocess.run(["xdotool", *args], env={**os.environ, "DISPLAY": self.display},
                                capture_output=True, text=True, check=False)
        if check and result.returncode != 0:
            raise RuntimeError(f"xdotool {' '.join(args)}: {result.stderr.strip()}")
        return result.stdout

    def click(self, x: int, y: int, button: int = 1, pause: float = 0.6) -> None:
        self.xdo("mousemove", str(x), str(y), "click", str(button))
        time.sleep(pause)

    def double(self, x: int, y: int, pause: float = 0.6) -> None:
        self.xdo("mousemove", str(x), str(y), "click", "--repeat", "2", "--delay", "80", "1")
        time.sleep(pause)

    def move(self, x: int, y: int, pause: float = 0.4) -> None:
        self.xdo("mousemove", str(x), str(y))
        time.sleep(pause)

    def key(self, *keys: str, pause: float = 0.5) -> None:
        """Aperta cada combinação (`ctrl+shift+F9`) pelo XTEST, por keycode.

        O `xdotool key` (3.20160805) manda `Alt_L` junto de toda tecla F no
        mapa do Xvfb: `ctrl+F9` virava Ctrl+Alt+F9, a troca de terminal que o
        servidor consome, e o atalho "não disparava" (59 §7.1.7, F8).
        """
        for combo in keys:
            _press_combo(self.display, combo)
            time.sleep(0.06)
        time.sleep(pause)

    def type(self, text: str, pause: float = 0.5) -> None:
        self.xdo("type", "--delay", "25", text)
        time.sleep(pause)

    def wait(self, seconds: float) -> None:
        time.sleep(seconds)

    def shot(self, name: str) -> Path:
        from PIL import ImageGrab

        self.count += 1
        path = self.shots / f"{self.count:02d}-{name}.png"
        ImageGrab.grab(xdisplay=self.display).save(path)
        return path

    def pixel(self, x: int, y: int) -> tuple[int, int, int]:
        from PIL import ImageGrab

        return ImageGrab.grab(xdisplay=self.display).getpixel((x, y))

    def close_window(self, timeout: float = 15.0) -> int | None:
        """Fecha pelo botão × da janela, como a pessoa faria; devolve a saída."""
        width = int(self.size.split("x")[0])
        self.click(width - 17, 19, pause=0.2)
        deadline = time.monotonic() + timeout
        while self.app is not None and self.app.poll() is None and time.monotonic() < deadline:
            time.sleep(0.2)
        return None if self.app is None else self.app.poll()

    def alive(self) -> bool:
        return self.app is not None and self.app.poll() is None

    def stop(self) -> int | None:
        code = None
        if self.app is not None:
            if self.app.poll() is None:
                self.app.terminate()
                try:
                    self.app.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    self.app.kill()
            code = self.app.returncode
        if self.xvfb is not None:
            self.xvfb.terminate()
            self.xvfb.wait(timeout=10)
        shutil.copy(self.log, self.shots / "ide.log")
        shutil.rmtree(self.base, ignore_errors=True)
        return code
