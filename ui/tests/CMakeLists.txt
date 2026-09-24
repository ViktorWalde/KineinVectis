# TESTES C++ da UI (2026-09-24).
#
# O projeto media Rust e QML e nao media C++: nenhuma decisao registrada
# justificava isso — era omissao, e o `verificar-exercitacao.sh` chegava a
# anotar "zero testes declarados" ao rodar o `ctest` real contra o proprio
# repositorio.
#
# So' entra aqui o que e' testavel sem janela. As 34 fontes do modulo QML ficam
# de fora POR ORA porque seis cabecalhos registram tipo QML (`QML_ELEMENT`,
# inclusive o `CoreClient`) e tira-los do `qt_add_qml_module` quebraria o
# registro. A fronteira cresce quando uma unidade puder sair sem quebrar isso —
# nao por decreto de uma vez.
find_package(Qt6 6.4 REQUIRED COMPONENTS Test)

function(kinein_add_cpp_test nome)
    add_executable("${nome}" "${nome}.cpp")
    kinein_enable_strict_compiler_options("${nome}")
    target_link_libraries("${nome}" PRIVATE kinein-ui-puro Qt6::Core Qt6::Test)
    target_include_directories("${nome}" PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../src")
    add_test(NAME "${nome}" COMMAND "${nome}")
    # Sem display: o teste mede decisao, nao pixel.
    set_tests_properties("${nome}" PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
endfunction()

kinein_add_cpp_test(tst_cli_args)
