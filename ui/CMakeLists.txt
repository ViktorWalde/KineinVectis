# Qt/QML frontend of Kinein Vectis.
# Presentation only: every action is sent to kinein-core over JSON-RPC.

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick)

qt_standard_project_setup()

qt_add_executable(kinein-vectis
    src/main.cpp
)

set_source_files_properties(qml/Theme.qml PROPERTIES QT_QML_SINGLETON_TYPE TRUE)

qt_add_qml_module(kinein-vectis
    URI KineinVectis
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/Theme.qml
        qml/FolderPickerDialog.qml
    SOURCES
        src/core_client.cpp
        src/core_client.h
        src/editor_highlighter.cpp
        src/editor_highlighter.h
    RESOURCES
        assets/app-icon.png
)

kinein_enable_strict_compiler_options(kinein-vectis)

# The generated qmltyperegistrations.cpp includes headers by base name.
target_include_directories(kinein-vectis PRIVATE src)

target_link_libraries(kinein-vectis PRIVATE Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick)
