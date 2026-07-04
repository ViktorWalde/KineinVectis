# Root build for the native (Qt/QML) side of Kernwerk Studio.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.28)

project(kernwerk-studio VERSION 0.1.0 LANGUAGES CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KernwerkStrictOptions)

add_subdirectory(ui)
