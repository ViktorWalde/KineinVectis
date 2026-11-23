# GoogleTest no core (docs/engenharia/estrategia-de-testes.md).
#
# Decisao do Viktor de 2026-11-22 (P06): GoogleTest 1.17.0, licenca BSD-3-Clause.
#
# Compilado aqui, com o perfil, e nao o do sistema:
# - no perfil debug-stl, o _GLIBCXX_DEBUG muda o ABI dos containers (manual da
#   libstdc++, Debug Mode), e a biblioteca do framework precisa estar no mesmo
#   modo do teste. O teste unit.TestFramework.* confere isso no link;
# - nos sanitizers, o codigo do framework tambem fica instrumentado.
#
# O tarball vem com hash fixado. Sem rede, aponte para uma copia local:
#   -DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=/caminho/googletest-1.17.0
include_guard(GLOBAL)

include(FetchContent)

set(INSTALL_GTEST OFF CACHE BOOL "O core nao instala o GoogleTest" FORCE)
set(BUILD_GMOCK ON CACHE BOOL "gMock para os dubles de porta" FORCE)

FetchContent_Declare(googletest
    URL https://github.com/google/googletest/releases/download/v1.17.0/googletest-1.17.0.tar.gz
    URL_HASH SHA256=65fab701d9829d38cb77c14acdc431d2108bfdbf8979e40eb8ae567edf10b27c
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    # Os cabecalhos entram como -isystem: os avisos do framework nao sao nossos.
    SYSTEM
)
FetchContent_MakeAvailable(googletest)

foreach(target IN ITEMS gtest gtest_main gmock gmock_main)
    kinein_apply_profile("${target}" THIRD_PARTY)
endforeach()

include(GoogleTest)

# kinein_add_unit_test(<nome> SOURCES <arquivos...> [LIBRARIES <alvos...>])
#
# Um executavel por modulo, com o perfil de teste, e cada TEST vira um teste do
# CTest com o prefixo "unit.". A descoberta roda na hora do teste (PRE_TEST),
# e nao no build: o binario so executa sob o ambiente do sanitizer.
function(kinein_add_unit_test name)
    cmake_parse_arguments(PARSE_ARGV 1 arg "" "" "SOURCES;LIBRARIES")
    if(NOT arg_SOURCES)
        message(FATAL_ERROR "kinein_add_unit_test(${name}): faltam as SOURCES")
    endif()
    add_executable("${name}" ${arg_SOURCES})
    kinein_apply_profile("${name}" TEST)
    target_link_libraries("${name}" PRIVATE GTest::gtest_main ${arg_LIBRARIES})
    gtest_discover_tests("${name}"
        TEST_PREFIX "unit."
        DISCOVERY_MODE PRE_TEST
        PROPERTIES ENVIRONMENT "${KINEIN_SANITIZER_ENVIRONMENT}"
    )
endfunction()
