# Root build for the native (Qt/QML) side of Kinein Vectis.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.25)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

project(kinein-vectis VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinStrictOptions)

add_subdirectory(ui)

include(FetchContent)
FetchContent_Declare(
    sqlite3
             GIT_REPOSITORY https://github.com/sqlite/sqlite
    GIT_TAG version-3.53.4
)
         FetchContent_MakeAvailable(sqlite3)

include(FetchContent)
FetchContent_Declare(
    asio
             GIT_REPOSITORY https://github.com/chriskohlhoff/asio
    GIT_TAG asio-1-38-2
)
         FetchContent_MakeAvailable(asio)

find_package(fmt CONFIG REQUIRED)
