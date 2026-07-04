# Kernwerk Studio — C++ Dependency Management

> Este documento define como o Kernwerk deve tratar dependências C/C++ de forma profissional e pragmática.

---

## 1. Objetivo

C++ possui múltiplas formas de dependência. O Kernwerk deve suportar as mais comuns sem criar um gerenciador próprio cedo demais.

---

## 2. Estratégia inicial

MVP:

```text
system packages;
find_package;
pkg-config;
FetchContent com cautela;
submodules apenas se usuário escolher.
```

Futuro:

```text
vcpkg;
Conan;
Meson;
CPM.cmake opcional;
package manager visual.
```

Regra:

```text
Não criar gerenciador de pacotes próprio.
```

---

## 3. find_package

Para bibliotecas CMake modernas:

```cmake
find_package(Qt6 REQUIRED COMPONENTS Core Widgets)
target_link_libraries(app PRIVATE Qt6::Core Qt6::Widgets)
```

Kernwerk deve detectar:

```text
pacote encontrado;
pacote ausente;
versão;
componentes;
targets importados.
```

---

## 4. pkg-config

Importante para GTK, GLib, system libs.

Exemplo:

```bash
pkg-config --modversion gtk4
pkg-config --cflags gtk4
pkg-config --libs gtk4
```

CMake:

```cmake
find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 REQUIRED IMPORTED_TARGET gtk4)
target_link_libraries(app PRIVATE PkgConfig::GTK4)
```

---

## 5. FetchContent

Pode ser útil, mas exige cuidado.

Permitido:

```text
bibliotecas pequenas;
dependências fixadas por tag/commit;
uso explícito;
sem baixar coisa oculta sem confirmação.
```

Exigir:

```text
URL visível;
versão/tag;
licença;
cache;
confirmação.
```

---

## 6. vcpkg futuro

Vantagens:

```text
pacotes C++ populares;
integração CMake;
manifest mode.
```

Não MVP porque adiciona complexidade.

---

## 7. Conan futuro

Vantagens:

```text
controle de toolchain;
build profiles;
binários;
projetos C++ maiores.
```

Não MVP.

---

## 8. UI de dependências

Painel futuro:

```text
Dependencies
├── System
│   ├── Qt6 found
│   └── GTK4 missing
├── CMake Packages
├── pkg-config
├── FetchContent
├── vcpkg futuro
└── Conan futuro
```

---

## 9. Licenças

Toda dependência deve mostrar licença quando possível.

Campos:

```text
nome;
versão;
origem;
licença;
método;
link;
status.
```

---

## 10. Segurança

A IDE não deve baixar dependências automaticamente sem confirmação.

Ações que exigem confirmação:

```text
adicionar FetchContent;
rodar package manager;
alterar CMakeLists;
baixar código externo;
executar script de build externo.
```

---

## 11. Cross-compilation

Em cross, dependências devem respeitar:

```text
sysroot;
CMAKE_FIND_ROOT_PATH;
pkg-config sysroot;
toolchain file;
target architecture.
```

Regra:

```text
Não misturar biblioteca do host com target em cross-compilation.
```

---

## 12. Critérios de aceite

MVP pronto quando:

```text
detecta find_package básico;
detecta pkg-config básico;
explica pacote ausente;
gera snippet CMake com confirmação;
não baixa dependência sem confirmar.
```

---

## 13. Decisão final

Para MVP, C++ dependencies devem ser simples e auditáveis.

Primeiro: system packages + find_package + pkg-config.  
Depois: vcpkg/Conan.
