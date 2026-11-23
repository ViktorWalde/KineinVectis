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
endif()
