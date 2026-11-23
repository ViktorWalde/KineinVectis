# Core da Kinein Vectis em C++23 (docs/arquitetura/fronteiras-e-responsabilidades.md).
#
# Projeto proprio, configurado a partir desta pasta (core/CMakePresets.json).
# Nao entra no build da UI de hoje: a UI e o core em Rust seguem como estao
# ate o core em C++ passar nos criterios de troca (docs/adr/ADR-0012).
cmake_minimum_required(VERSION 3.25)

project(kinein_core LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinProfile)

# clang-tidy no build (preset lint). O --extra-arg tira o ruido das flags que
# so o GCC conhece quando alguem o liga com o GCC; o preset lint usa o Clang.
option(KINEIN_CLANG_TIDY "Roda o clang-tidy em cada unidade compilada" OFF)
if(KINEIN_CLANG_TIDY)
    find_program(KINEIN_CLANG_TIDY_EXE NAMES clang-tidy REQUIRED)
    set(CMAKE_CXX_CLANG_TIDY "${KINEIN_CLANG_TIDY_EXE}" "--extra-arg=-Wno-unknown-warning-option")
endif()

enable_testing()

add_subdirectory(lib)

# O adaptador Qt e' opcional: o build sem ele nem procura o Qt, e assim nada em
# core/lib consegue depender do Qt sem quebrar o preset dev.
option(KINEIN_CORE_WITH_QT "Compila o adaptador Qt (core/qt)" OFF)
if(KINEIN_CORE_WITH_QT)
    add_subdirectory(qt)
endif()

add_subdirectory(tests)
