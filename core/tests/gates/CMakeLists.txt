# Effectiveness measurement of the project's own gates.
#
# Documented in: docs/engineering/gates-and-effectiveness.md
#
# Each gate has a versioned corpus: planted defects it must reject and correct
# forms it must accept. The measurement runs again on every build of the preset
# that has the gate's tool, so no edit to a gate passes without being measured.

# clang-tidy gates: only in the lint preset, which has clang-tidy.
if(KINEIN_CLANG_TIDY)
    find_package(Python3 3.10 REQUIRED COMPONENTS Interpreter)

    # Processes only through the launcher (docs/architecture/processes-and-trust.md).
    add_test(NAME gate.launcher
        COMMAND "${Python3_EXECUTABLE}" -I "${CMAKE_CURRENT_SOURCE_DIR}/tidy_corpus.py"
            --clang-tidy "${KINEIN_CLANG_TIDY_EXE}"
            --config "${PROJECT_SOURCE_DIR}/.clang-tidy"
            --check bugprone-unsafe-functions
            --std "${KINEIN_CXX_STANDARD}"
            --reject "${CMAKE_CURRENT_SOURCE_DIR}/launcher/forbidden-processes.cpp"
            --accept "${CMAKE_CURRENT_SOURCE_DIR}/launcher/allowed-processes.cpp")

    # No detached thread (CP.26), through the same clang-tidy option.
    add_test(NAME gate.detach
        COMMAND "${Python3_EXECUTABLE}" -I "${CMAKE_CURRENT_SOURCE_DIR}/tidy_corpus.py"
            --clang-tidy "${KINEIN_CLANG_TIDY_EXE}"
            --config "${PROJECT_SOURCE_DIR}/.clang-tidy"
            --check bugprone-unsafe-functions
            --std "${KINEIN_CXX_STANDARD}"
            --reject "${CMAKE_CURRENT_SOURCE_DIR}/threads/detach-forbidden.cpp"
            --accept "${CMAKE_CURRENT_SOURCE_DIR}/threads/detach-allowed.cpp")

    # Duplication (P20): the detector built on libclang, measured against the
    # duplication/ corpus. It uses the libclang and the internal headers of the
    # same Clang that compiles; without them configure stops, and the gate is
    # never skipped silently.
    if(NOT CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
        message(FATAL_ERROR "gate.duplication: the clang-tidy preset compiles with Clang")
    endif()
    execute_process(
        COMMAND "${Python3_EXECUTABLE}" -I -c "import clang.cindex"
        RESULT_VARIABLE kinein_python_clang
        OUTPUT_QUIET ERROR_QUIET)
    if(NOT kinein_python_clang EQUAL 0)
        message(FATAL_ERROR
            "gate.duplication: the libclang Python bindings are missing "
            "(python3-clang on Fedora, python3-clang-<Clang version> on Ubuntu)")
    endif()
    execute_process(
        COMMAND "${CMAKE_CXX_COMPILER}" -print-resource-dir
        OUTPUT_VARIABLE KINEIN_CLANG_RESOURCE_DIR
        OUTPUT_STRIP_TRAILING_WHITESPACE
        COMMAND_ERROR_IS_FATAL ANY)
    # <prefix>/lib/clang/<version> -> <prefix>/lib, where Ubuntu puts the
    # libclang of that version (/usr/lib/llvm-22/lib). On Fedora it sits in
    # the default path (/usr/lib64).
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
