# Perfil de compilacao do core em C++ (docs/engenharia/perfis-de-compilacao-e-hardening.md).
#
# O perfil e' o unico dono das flags de otimizacao, depuracao, avisos,
# endurecimento e sanitizers. O CMAKE_BUILD_TYPE nao decide nada aqui: as flags
# padrao do CMake por configuracao sao zeradas neste escopo.
include_guard(GLOBAL)

set(KINEIN_PROFILES dev production asan-ubsan tsan debug-stl)
set(KINEIN_PROFILE "dev" CACHE STRING "Perfil de compilacao: ${KINEIN_PROFILES}")
set_property(CACHE KINEIN_PROFILE PROPERTY STRINGS ${KINEIN_PROFILES})
if(NOT KINEIN_PROFILE IN_LIST KINEIN_PROFILES)
    message(FATAL_ERROR "KINEIN_PROFILE='${KINEIN_PROFILE}' nao existe. Use um de: ${KINEIN_PROFILES}")
endif()

# C++23 sem extensoes. O 26 so existe para o job de sondagem (docs/engenharia/regras-de-cpp23.md).
set(KINEIN_CXX_STANDARD "23" CACHE STRING "Padrao da linguagem: 23; 26 so na sondagem")
set_property(CACHE KINEIN_CXX_STANDARD PROPERTY STRINGS 23 26)
if(NOT KINEIN_CXX_STANDARD MATCHES "^(23|26)$")
    message(FATAL_ERROR "KINEIN_CXX_STANDARD='${KINEIN_CXX_STANDARD}': use 23 ou 26")
endif()
set(CMAKE_CXX_STANDARD ${KINEIN_CXX_STANDARD})
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Versoes minimas, decisao do Viktor de 2026-11-22: GCC 14 e Clang 20, o piso
# que o CI testa no ubuntu-26.04. Antes do Clang 19, o __cpp_concepts do Clang
# era 201907L, e a libstdc++ so declara o std::expected com 202002L (PR #87998
# do LLVM): com Clang 18 o core nao compila. Abaixo do piso, o configure falha.
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" AND CMAKE_CXX_COMPILER_VERSION VERSION_LESS 14)
    message(FATAL_ERROR "GCC ${CMAKE_CXX_COMPILER_VERSION}: o core exige GCC 14 ou mais novo")
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "Clang" AND CMAKE_CXX_COMPILER_VERSION VERSION_LESS 20)
    message(FATAL_ERROR "Clang ${CMAKE_CXX_COMPILER_VERSION}: o core exige Clang 20 ou mais novo")
elseif(NOT CMAKE_CXX_COMPILER_ID MATCHES "^(GNU|Clang|MSVC)$")
    message(FATAL_ERROR "Compilador ${CMAKE_CXX_COMPILER_ID} sem perfil definido")
endif()

foreach(config DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
    set(CMAKE_CXX_FLAGS_${config} "")
endforeach()

# PIE em tudo; o CMake poe o -pie no link dos executaveis (politica CMP0083).
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
include(CheckPIESupported)
check_pie_supported(LANGUAGES CXX)

include(KineinWarnings)
include(KineinHardening)
include(KineinSanitizers)

# Otimizacao e depuracao por perfil. O FORTIFY exige -O1 ou mais (guia OpenSSF).
function(kinein_apply_optimization target)
    if(MSVC)
        if(KINEIN_PROFILE STREQUAL "debug-stl")
            target_compile_options("${target}" PRIVATE /Od /Zi)
        else()
            target_compile_options("${target}" PRIVATE /O2 /Zi)
        endif()
        return()
    endif()

    if(KINEIN_PROFILE MATCHES "^(dev|production)$")
        target_compile_options("${target}" PRIVATE -O2 -g)
    elseif(KINEIN_PROFILE MATCHES "^(asan-ubsan|tsan)$")
        target_compile_options("${target}" PRIVATE -O1 -g -fno-omit-frame-pointer)
    elseif(KINEIN_PROFILE STREQUAL "debug-stl")
        target_compile_options("${target}" PRIVATE -O0 -g3)
    endif()
endfunction()

# kind: LIBRARY (biblioteca do core), EXECUTABLE (processo entregue), TEST, ou
# THIRD_PARTY (codigo de fora compilado junto, como o GoogleTest).
#
# THIRD_PARTY recebe tudo o que muda o ABI ou a instrumentacao (otimizacao,
# modo da biblioteca padrao, sanitizers, protecoes) e nada dos avisos nem do
# clang-tidy: o codigo nao e' nosso, e o -Werror nele so quebraria o build
# quando o compilador mudasse.
function(kinein_apply_profile target kind)
    if(NOT kind MATCHES "^(LIBRARY|EXECUTABLE|TEST|THIRD_PARTY)$")
        message(FATAL_ERROR
            "kinein_apply_profile(${target} ${kind}): use LIBRARY, EXECUTABLE, TEST ou THIRD_PARTY")
    endif()
    kinein_apply_optimization("${target}")
    if(kind STREQUAL "THIRD_PARTY")
        # Aviso no codigo de fora nao tem acao possivel aqui: medido em
        # 2026-11-22, o Clang 23 avisa -Wcharacter-conversion (ligado por
        # padrao) no gtest-printers.h do GoogleTest 1.17.0. Para quem usa o
        # framework, os cabecalhos ja entram como -isystem.
        if(MSVC)
            target_compile_options("${target}" PRIVATE /w)
        else()
            target_compile_options("${target}" PRIVATE -w)
        endif()
        set_target_properties("${target}" PROPERTIES CXX_CLANG_TIDY "")
    else()
        kinein_apply_warnings("${target}" "${kind}")
    endif()
    kinein_apply_hardening("${target}" "${kind}")
    kinein_apply_sanitizers("${target}")
endfunction()

message(STATUS "Kinein core: perfil ${KINEIN_PROFILE}, C++${KINEIN_CXX_STANDARD}, "
               "${CMAKE_CXX_COMPILER_ID} ${CMAKE_CXX_COMPILER_VERSION}")
