# Avisos do compilador para o core (docs/engenharia/perfis-de-compilacao-e-hardening.md).
#
# As listas sao fixas por compilador e nao passam por check_cxx_compiler_flag:
# flag desconhecida quebra o build em vez de sumir em silencio (falha fechado).
# Base: o KineinStrictOptions.cmake da 0.3, que ja compilava a UI com -Werror,
# mais o TL;DR do guia OpenSSF de 2026-08-20.
include_guard(GLOBAL)

option(KINEIN_WERROR "Aviso vira erro. Ligado nos perfis de desenvolvimento e CI; nunca na distribuicao do fonte" OFF)

set(KINEIN_WARNINGS_COMMON
    -Wall
    -Wextra
    -Wpedantic
    -Wconversion
    -Wsign-conversion
    -Wshadow
    -Wnon-virtual-dtor
    -Wold-style-cast
    -Wcast-align
    -Wcast-qual
    -Wunused
    -Woverloaded-virtual
    -Wnull-dereference
    -Wdouble-promotion
    -Wformat
    -Wformat=2
    -Werror=format-security
    -Wimplicit-fallthrough
    -Wundef
    -Wextra-semi
    -Wzero-as-null-pointer-constant
    -Wredundant-decls
    -Wswitch-enum
    -Wfloat-equal
    -Wsuggest-override
    -Wvla
    -Wdate-time
    -Wdeprecated
)

set(KINEIN_WARNINGS_GCC
    -Wmissing-declarations
    -Wduplicated-branches
    -Wduplicated-cond
    -Wlogical-op
    -Wuseless-cast
    -Wdangling-reference
    -Wtrampolines
    -Wformat-overflow=2
    -Wformat-truncation=2
    -Wcast-align=strict
    -Walloca
    -Wimplicit-fallthrough=5
)

set(KINEIN_WARNINGS_CLANG
    -Wmissing-prototypes
    -Wcomma
    -Wloop-analysis
    -Wmissing-noreturn
    -Wunreachable-code-aggressive
    -Wheader-hygiene
    -Wctad-maybe-unsupported
    -Wthread-safety
)

# So na biblioteca do core: inicializacao e destruicao estatica fora de ordem.
# Teste fica de fora porque o registro de casos usa construtor global.
set(KINEIN_WARNINGS_CLANG_LIBRARY
    -Wglobal-constructors
    -Wexit-time-destructors
)

# Herdado do KineinStrictOptions.cmake (60 §3.2); nao conferido nesta maquina.
set(KINEIN_WARNINGS_MSVC
    /W4
    /permissive-
    /utf-8
    /Zc:__cplusplus
    /Zc:preprocessor
)

function(kinein_apply_warnings target kind)
    if(MSVC)
        target_compile_options("${target}" PRIVATE ${KINEIN_WARNINGS_MSVC})
        if(KINEIN_WERROR)
            target_compile_options("${target}" PRIVATE /WX)
        endif()
        return()
    endif()

    target_compile_options("${target}" PRIVATE ${KINEIN_WARNINGS_COMMON})
    if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
        target_compile_options("${target}" PRIVATE ${KINEIN_WARNINGS_GCC})
    elseif(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
        target_compile_options("${target}" PRIVATE ${KINEIN_WARNINGS_CLANG})
        if(kind STREQUAL "LIBRARY")
            target_compile_options("${target}" PRIVATE ${KINEIN_WARNINGS_CLANG_LIBRARY})
        endif()
    endif()

    if(KINEIN_WERROR)
        target_compile_options("${target}" PRIVATE -Werror)
    endif()
endfunction()
