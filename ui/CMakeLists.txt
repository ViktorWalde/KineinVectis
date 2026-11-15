cmake_minimum_required(VERSION 3.21)
project(kinein_prova_ui LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.7 REQUIRED COMPONENTS Gui Network Quick)
qt_standard_project_setup(REQUIRES 6.7)

qt_add_executable(kinein-prova-ui
    main.cpp
    msgpack.cpp msgpack.h
    nvim_rpc.cpp nvim_rpc.h
    supervisor_link.cpp supervisor_link.h
    prova_control.cpp prova_control.h
)

qt_add_qml_module(kinein-prova-ui
    URI KineinProva
    VERSION 1.0
    QML_FILES Main.qml
    SOURCES nvim_view.cpp nvim_view.h
)

target_compile_options(kinein-prova-ui PRIVATE -Wall -Wextra)
target_link_libraries(kinein-prova-ui PRIVATE Qt6::Gui Qt6::Network Qt6::Quick)
