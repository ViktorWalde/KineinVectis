# Adaptador Qt do core: liga a biblioteca ao laco de eventos e ao transporte.
# Fino de proposito: nenhuma regra de dominio mora aqui
# (docs/arquitetura/fronteiras-e-responsabilidades.md).
#
# O moc so roda nos alvos desta pasta; o codigo de core/lib nunca passa por ele.
# A versao do Qt e' a da UI (cmake/KineinQt.cmake da raiz).
find_package(Qt6 6.12 REQUIRED COMPONENTS Core)

# Ainda sem codigo. Com o primeiro .cpp: add_library(... STATIC), AUTOMOC ON
# neste alvo e kinein_apply_profile(kinein_core_qt LIBRARY).
add_library(kinein_core_qt INTERFACE)
add_library(kinein::core_qt ALIAS kinein_core_qt)
target_link_libraries(kinein_core_qt INTERFACE kinein::core_lib Qt6::Core)
