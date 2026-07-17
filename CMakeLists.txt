# Root build for the native (Qt/QML) side of Kinein Vectis.
# The Rust workspace in crates/ is built separately with Cargo.
cmake_minimum_required(VERSION 3.25)

project(kinein-vectis VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
include(KineinStrictOptions)

# Testes de C++ da UI. `BUILD_TESTING` e' do CTest e vem ON por padrao; o
# AppImage passa OFF para nao carregar Qt6::Test na entrega.
include(CTest)

add_subdirectory(ui)
