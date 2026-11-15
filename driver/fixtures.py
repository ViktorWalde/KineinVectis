"""As fixtures da prova: projetos de verdade, compilados de verdade."""

import os
import stat
import subprocess
import textwrap

# A geradora da fixture de desempenho é a do produto na linha de base
# (base/arvore, o 308c1561 com a instrumentação), não a de um checkout qualquer.
REPO = os.environ.get("KINEIN_REPO",
                      os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "base", "arvore"))


def escrever(raiz, rel, conteudo, executavel=False):
    caminho = os.path.join(raiz, rel)
    os.makedirs(os.path.dirname(caminho), exist_ok=True)
    with open(caminho, "w") as f:
        f.write(conteudo)
    if executavel:
        os.chmod(caminho, os.stat(caminho).st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return caminho


# --- C++ (CMake, dois presets) -----------------------------------------------

CPP_MAIN = textwrap.dedent("""\
    #include <cstdio>
    #include <vector>

    #include "util.h"

    int main() {
        const Ponto p{3, 4};
        std::printf("%d\\n", soma(p));
        return 0;
    }
    """)

CPP_UTIL_H = textwrap.dedent("""\
    #pragma once

    struct Ponto {
        int x;
        int y;
    };

    int soma(const Ponto& p);
    """)

CPP_UTIL_CPP = textwrap.dedent("""\
    #include "util.h"

    int soma(const Ponto& p) {
        return p.x + p.y;
    }
    """)

# Só compila em C++20: o clangd com o preset de C++17 tem de reclamar.
CPP_PADRAO = textwrap.dedent("""\
    #include <span>

    consteval int dobro(int v) {
        return v * 2;
    }

    int total(std::span<const int> valores) {
        int t = dobro(0);
        for (int v : valores) {
            t += v;
        }
        return t;
    }
    """)

CMAKE = textwrap.dedent("""\
    cmake_minimum_required(VERSION 3.21)
    project(prova_cpp CXX)
    if(NOT CMAKE_CXX_STANDARD)
      set(CMAKE_CXX_STANDARD 20)
    endif()
    set(CMAKE_CXX_STANDARD_REQUIRED ON)
    add_executable(app src/main.cpp src/util.cpp)
    add_library(padrao OBJECT src/padrao.cpp)
    """)


def projeto_cpp(raiz):
    escrever(raiz, "CMakeLists.txt", CMAKE)
    escrever(raiz, "src/main.cpp", CPP_MAIN)
    escrever(raiz, "src/util.h", CPP_UTIL_H)
    escrever(raiz, "src/util.cpp", CPP_UTIL_CPP)
    escrever(raiz, "src/padrao.cpp", CPP_PADRAO)
    # O "core" configurou os dois presets: o que muda entre eles é o padrão.
    for padrao in ("17", "20"):
        subprocess.run(["cmake", "-S", raiz, "-B", os.path.join(raiz, f"build-{padrao}"), "-G", "Ninja",
                        f"-DCMAKE_CXX_STANDARD={padrao}", "-DCMAKE_EXPORT_COMPILE_COMMANDS=ON",
                        "-DCMAKE_CXX_COMPILER=g++"],
                       check=True, capture_output=True)
    return raiz


# --- Rust (Cargo) -------------------------------------------------------------

RUST_MAIN = textwrap.dedent("""\
    mod util;

    fn main() {
        let valores: Vec<i32> = vec![1, 2, 3];
        println!("{}", util::dobro(valores.len() as i32));
    }
    """)

RUST_UTIL = textwrap.dedent("""\
    pub fn dobro(v: i32) -> i32 {
        v * 2
    }
    """)


def projeto_rust(raiz):
    escrever(raiz, "Cargo.toml",
             '[package]\nname = "prova_rust"\nversion = "0.1.0"\nedition = "2021"\n\n[dependencies]\n')
    escrever(raiz, "src/main.rs", RUST_MAIN)
    escrever(raiz, "src/util.rs", RUST_UTIL)
    return raiz


# --- Desempenho: a geradora do medir-core.py -----------------------------------

def fixture_desempenho(raiz, funcoes):
    os.makedirs(raiz, exist_ok=True)
    saida = subprocess.run(["python3", os.path.join(REPO, "scripts", "medir-core.py"), "--emit-fixture", raiz,
                            str(funcoes)], check=True, capture_output=True, text=True).stdout
    dados = dict(l.split("=", 1) for l in saida.split() if "=" in l)
    return dados["fixture_path"], int(dados["fixture_lines"])


# --- O projeto hostil (64 §2.1 e 65 §8) ----------------------------------------

def projeto_hostil(raiz, marcas):
    """Cada vetor, se executar, cria um arquivo em `marcas`."""
    os.makedirs(marcas, exist_ok=True)

    def script(nome):
        return f"#!/bin/sh\ntouch '{marcas}/{nome}'\n"

    escrever(raiz, ".venv/bin/python", script("venv-python"), executavel=True)
    escrever(raiz, "ferramentas/cc-hostil", script("query-driver") + 'exec /usr/bin/cc "$@"\n', executavel=True)
    escrever(raiz, "ferramentas/rustc-wrapper", script("rustc-wrapper") + 'shift\nexec rustc "$@"\n',
             executavel=True)
    escrever(raiz, "ferramentas/fsmonitor", script("fsmonitor"), executavel=True)
    escrever(raiz, ".cargo/config.toml", f'[build]\nrustc-wrapper = "{raiz}/ferramentas/rustc-wrapper"\n')
    escrever(raiz, "Cargo.toml", '[package]\nname = "hostil"\nversion = "0.1.0"\nedition = "2021"\n')
    escrever(raiz, "build.rs", f'fn main() {{ std::fs::write("{marcas}/build-rs", "x").unwrap(); }}\n')
    escrever(raiz, "src/main.rs", "fn main() {\n    println!(\"oi\");\n}\n")
    escrever(raiz, "src/hostil.c", "int main(void) {\n    return 0;\n}\n")
    escrever(raiz, "compile_commands.json",
             f'[{{"directory": "{raiz}", "command": "{raiz}/ferramentas/cc-hostil -c src/hostil.c",'
             f' "file": "{raiz}/src/hostil.c"}}]\n')
    escrever(raiz, ".kinein/toolchain.json", f'{{"compiler": "{raiz}/ferramentas/cc-hostil"}}\n')
    # Os vetores do Neovim: configuração local, modeline e uma biblioteca nativa
    # com construtor, posta onde um runtimepath aberto a carregaria.
    lua_marca = lambda nome: f"vim.fn.writefile({{'x'}}, '{marcas}/{nome}')\n"
    escrever(raiz, ".nvim.lua", lua_marca("nvim-lua-local"))
    escrever(raiz, ".exrc", f"call writefile(['x'], '{marcas}/exrc')\n")
    escrever(raiz, "lua/kinein/init.lua", lua_marca("lua-do-projeto"))
    escrever(raiz, "plugin/hostil.lua", lua_marca("plugin-do-projeto"))
    escrever(raiz, "src/modeline.txt",
             "texto\n/* vim: set tabstop=13 shiftwidth=13 foldexpr=writefile(['x'],'"
             f"{marcas}/modeline') : */\n")
    so_fonte = escrever(raiz, "ferramentas/construtor.c",
                        "#include <fcntl.h>\n#include <unistd.h>\n"
                        f"__attribute__((constructor)) static void marca(void) {{ close(creat(\"{marcas}/biblioteca\", 0644)); }}\n")
    for destino in ("parser/c.so", "parser/rust.so", "parser/cpp.so", "kinein.so", "ffi.so"):
        os.makedirs(os.path.dirname(os.path.join(raiz, destino)) or raiz, exist_ok=True)
        subprocess.run(["cc", "-shared", "-fPIC", "-o", os.path.join(raiz, destino), so_fonte], check=True)
    subprocess.run(["git", "init", "-q", raiz], check=True)
    subprocess.run(["git", "-C", raiz, "config", "core.fsmonitor", f"{raiz}/ferramentas/fsmonitor"], check=True)
    return raiz


def casa_plantada(raiz, marcas):
    """A configuração da pessoa, plantada: nada dela pode rodar."""
    lua_marca = lambda nome: f"vim.fn.writefile({{'x'}}, '{marcas}/{nome}')\n"
    escrever(raiz, ".config/nvim/init.lua", lua_marca("init-da-pessoa"))
    escrever(raiz, ".config/nvim/plugin/p.lua", lua_marca("plugin-da-pessoa"))
    escrever(raiz, ".local/share/nvim/site/pack/x/start/p/plugin/p.lua", lua_marca("pacote-da-pessoa"))
    escrever(raiz, ".local/share/nvim/site/plugin/s.lua", lua_marca("site-da-pessoa"))
    ambiente = {
        "HOME": raiz,
        "XDG_CONFIG_HOME": os.path.join(raiz, ".config"),
        "XDG_DATA_HOME": os.path.join(raiz, ".local/share"),
        "VIMINIT": f"lua vim.fn.writefile({{'x'}}, '{marcas}/viminit')",
        "VIMRUNTIME": os.path.join(raiz, "runtime-falso"),
        "NVIM_APPNAME": "hostil",
    }
    escrever(raiz, "runtime-falso/plugin/r.lua", lua_marca("vimruntime-falso"))
    return ambiente
