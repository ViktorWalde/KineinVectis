# Top-level CMake project of the Kinein Vectis C++23 core.
#
# Documented in: docs/architecture/boundaries-and-responsibilities.md
#
# A standalone project, configured from this directory (core/CMakePresets.json). It is not part
# of the UI build: the UI connects to the core at the sync of
# docs/adr/ADR-0012-cpp23-backend.md.
cmake_minimum_required(VERSION 3.25)

project(kinein_core LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinProfile)

# clang-tidy in the build (lint preset). The --extra-arg silences the flags that
# only GCC knows when someone enables it with GCC; the lint preset uses Clang.
option(KINEIN_CLANG_TIDY "Run clang-tidy on every compiled unit" OFF)
if(KINEIN_CLANG_TIDY)
    find_program(KINEIN_CLANG_TIDY_EXE NAMES clang-tidy REQUIRED)
    set(CMAKE_CXX_CLANG_TIDY "${KINEIN_CLANG_TIDY_EXE}" "--extra-arg=-Wno-unknown-warning-option")
endif()

enable_testing()

add_subdirectory(lib)

# The Qt adapter is optional: the build without it does not even look for Qt,
# so nothing in core/lib can depend on Qt without breaking the dev preset.
option(KINEIN_CORE_WITH_QT "Build the Qt adapter (core/qt)" OFF)
if(KINEIN_CORE_WITH_QT)
    add_subdirectory(qt)
endif()

add_subdirectory(tests)
