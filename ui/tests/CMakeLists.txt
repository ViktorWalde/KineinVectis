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

# O ALVO AGREGADO existe porque o `verificar-cpp-testes.sh` compilava um alvo
# pelo NOME (`tst_cli_args`). O segundo teste do projeto, em 2026-09-25, nao foi
# compilado por isso e o CTest o reportou como "Not Run" — o gate acusou, mas
# acusou a coisa errada. Agora cada teste entra aqui ao nascer, e o script
# compila UM alvo que os contem todos.
add_custom_target(kinein-cpp-tests)

function(kinein_add_cpp_test nome)
    add_executable("${nome}" "${nome}.cpp")
    kinein_enable_strict_compiler_options("${nome}")
    target_link_libraries("${nome}" PRIVATE kinein-ui-puro Qt6::Core Qt6::Test)
    target_include_directories("${nome}" PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../src")
    add_test(NAME "${nome}" COMMAND "${nome}")
    add_dependencies(kinein-cpp-tests "${nome}")
    # Sem display: o teste mede decisao, nao pixel.
    set_tests_properties("${nome}" PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
endfunction()

kinein_add_cpp_test(tst_cli_args)
kinein_add_cpp_test(tst_markdown_policy)
kinein_add_cpp_test(tst_log_redaction)
kinein_add_cpp_test(tst_single_instance)

# O clipboard é um singleton de UI com MIME do sistema. Compila a MESMA fonte
# que o aplicativo e cria QGuiApplication no teste, sem mockar QClipboard.
kinein_add_cpp_test(tst_clipboard_files)
target_sources(tst_clipboard_files PRIVATE ../src/clipboard.cpp ../src/clipboard.h)
target_link_libraries(tst_clipboard_files PRIVATE Qt6::Gui Qt6::Qml)
set_target_properties(tst_clipboard_files PROPERTIES AUTOMOC ON)
# O MOC do Qt 6.10 usa CTAD em codigo gerado e o Clang recente o diagnostica
# sob -Werror. A excecao vale so para as duas unidades que incluem esse MOC.
set_source_files_properties(
    tst_clipboard_files.cpp
    "${CMAKE_CURRENT_BINARY_DIR}/tst_clipboard_files_autogen/mocs_compilation.cpp"
    PROPERTIES COMPILE_OPTIONS -Wno-ctad-maybe-unsupported
)
