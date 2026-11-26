# Root build for the native (Qt/QML) side of Kinein Vectis.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.25)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(kinein-vectis VERSION 0.3.9 LANGUAGES CXX)
set(KINEIN_VERSION "${PROJECT_VERSION}" CACHE STRING "Versao exibida pela UI e pelo pacote")

# C++23 without extensions, as in the core (docs/engineering/cpp23-rules.md): every target, the
# ones Qt generates included, compiles with -std=c++23, never -std=gnu++23.
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinStrictOptions)
include(KineinQt)

# CTest, para os testes C++ da UI (2026-09-24). O projeto media Rust e QML e nao
# media C++; ver ui/tests/CMakeLists.txt.
include(CTest)

add_subdirectory(ui)
