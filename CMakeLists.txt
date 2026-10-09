# Root build for the native (Qt/QML) side of Kinein Vectis.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.25)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(kinein-vectis VERSION 0.3.5 LANGUAGES CXX)

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinStrictOptions)

# CTest, para os testes C++ da UI (2026-09-24). O projeto media Rust e QML e nao
# media C++; ver ui/tests/CMakeLists.txt.
include(CTest)

add_subdirectory(ui)
