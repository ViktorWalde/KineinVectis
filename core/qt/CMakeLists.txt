# Qt adapter target of the core: connects the library to the event loop and the transport.
#
# Documented in: docs/architecture/boundaries-and-responsibilities.md
#
# Thin on purpose: no domain rule lives here. moc runs only on the targets of this directory;
# core/lib code never goes through it. The Qt version is the UI's (cmake/KineinQt.cmake).
find_package(Qt6 6.12 REQUIRED COMPONENTS Core)

# No code yet. With the first .cpp: add_library(... STATIC), AUTOMOC ON
# on this target and kinein_apply_profile(kinein_core_qt LIBRARY).
add_library(kinein_core_qt INTERFACE)
add_library(kinein::core_qt ALIAS kinein_core_qt)
target_link_libraries(kinein_core_qt INTERFACE kinein::core_lib Qt6::Core)
