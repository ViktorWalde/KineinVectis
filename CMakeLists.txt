# Root build for the native (Qt/QML) side of Kinein Vectis.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.25)

project(kinein-vectis VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinStrictOptions)

add_subdirectory(ui)
