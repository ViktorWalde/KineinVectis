# Compiler warnings of the core.
#
# Documented in: docs/engineering/build-profiles-and-hardening.md
#
# The lists are fixed per compiler and do not go through check_cxx_compiler_flag: an unknown
# flag breaks the build instead of silently vanishing (fail closed). Base: the TL;DR of the
# OpenSSF Compiler Options Hardening Guide for C and C++, version of 2026-08-20. Not verified:
# the source of each flag outside that list in the manuals of GCC and Clang.
include_guard(GLOBAL)

option(KINEIN_WERROR "Turn warnings into errors. On in the development and CI profiles; never in the source distribution" OFF)

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
    -Wbidi-chars=any
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

# Core library only: static initialization and destruction order.
# Tests are left out because test case registration uses global constructors.
set(KINEIN_WARNINGS_CLANG_LIBRARY
    -Wglobal-constructors
    -Wexit-time-destructors
)

# Not verified: the reference machine has no MSVC.
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
