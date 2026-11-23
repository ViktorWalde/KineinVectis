# Medida de eficacia dos gates proprios (docs/engenharia/gates-e-medida-de-eficacia.md).
#
# Cada gate tem um corpus versionado: defeitos plantados que ele tem de reprovar
# e formas corretas que ele tem de aceitar. A medida roda de novo a cada build
# do preset que tem a ferramenta do gate, entao mudar o gate sem medir de novo
# nao passa.

# Gates do clang-tidy: so no preset lint, que tem o clang-tidy.
if(KINEIN_CLANG_TIDY)
    find_package(Python3 3.10 REQUIRED COMPONENTS Interpreter)

    # Processo so pelo lancador (docs/arquitetura/processos-e-confianca.md).
    add_test(NAME gate.launcher
        COMMAND "${Python3_EXECUTABLE}" -I "${CMAKE_CURRENT_SOURCE_DIR}/tidy_corpus.py"
            --clang-tidy "${KINEIN_CLANG_TIDY_EXE}"
            --config "${PROJECT_SOURCE_DIR}/.clang-tidy"
            --check bugprone-unsafe-functions
            --std "${KINEIN_CXX_STANDARD}"
            --reject "${CMAKE_CURRENT_SOURCE_DIR}/launcher/processos-proibidos.cpp"
            --accept "${CMAKE_CURRENT_SOURCE_DIR}/launcher/processos-permitidos.cpp")

    # Thread nunca solta (CP.26), pela mesma opcao do clang-tidy.
    add_test(NAME gate.detach
        COMMAND "${Python3_EXECUTABLE}" -I "${CMAKE_CURRENT_SOURCE_DIR}/tidy_corpus.py"
            --clang-tidy "${KINEIN_CLANG_TIDY_EXE}"
            --config "${PROJECT_SOURCE_DIR}/.clang-tidy"
            --check bugprone-unsafe-functions
            --std "${KINEIN_CXX_STANDARD}"
            --reject "${CMAKE_CURRENT_SOURCE_DIR}/threads/detach-proibido.cpp"
            --accept "${CMAKE_CURRENT_SOURCE_DIR}/threads/detach-permitido.cpp")

    # Duplicacao (P20): o detector sobre a libclang, medido contra o corpus
    # de duplication/. Usa a libclang e os cabecalhos internos do mesmo Clang
    # que compila; sem eles o configure para, e o gate nunca pula calado.
    if(NOT CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
        message(FATAL_ERROR "gate.duplication: o preset com clang-tidy compila com o Clang")
    endif()
    execute_process(
        COMMAND "${Python3_EXECUTABLE}" -I -c "import clang.cindex"
        RESULT_VARIABLE kinein_python_clang
        OUTPUT_QUIET ERROR_QUIET)
    if(NOT kinein_python_clang EQUAL 0)
        message(FATAL_ERROR
            "gate.duplication: faltam os bindings de Python da libclang "
            "(python3-clang no Fedora, python3-clang-<versao do Clang> no Ubuntu)")
    endif()
    execute_process(
        COMMAND "${CMAKE_CXX_COMPILER}" -print-resource-dir
        OUTPUT_VARIABLE KINEIN_CLANG_RESOURCE_DIR
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY)
    # <prefixo>/lib/clang/<versao> -> <prefixo>/lib, onde o Ubuntu poe a
    # libclang da versao (/usr/lib/llvm-22/lib). No Fedora ela fica no
    # caminho padrao (/usr/lib64).
    cmake_path(GET KINEIN_CLANG_RESOURCE_DIR PARENT_PATH kinein_clang_lib)
    cmake_path(GET kinein_clang_lib PARENT_PATH kinein_clang_lib)
    find_library(KINEIN_LIBCLANG NAMES clang HINTS "${kinein_clang_lib}" REQUIRED)
    add_test(NAME gate.duplication
        COMMAND "${Python3_EXECUTABLE}" -I "${CMAKE_CURRENT_SOURCE_DIR}/duplication/clones.py"
            --libclang "${KINEIN_LIBCLANG}"
            --resource-dir "${KINEIN_CLANG_RESOURCE_DIR}"
            --std "${KINEIN_CXX_STANDARD}"
            measure "${CMAKE_CURRENT_SOURCE_DIR}/duplication")
endif()
