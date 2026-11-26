# Build profile of the core.
#
# Documented in: docs/engineering/build-profiles-and-hardening.md
#
# The profile is the only owner of the optimization, debug, warning, hardening
# and sanitizer flags. CMAKE_BUILD_TYPE decides nothing here: CMake's default
# per-configuration flags are cleared in this scope.
include_guard(GLOBAL)

set(KINEIN_PROFILES dev production asan-ubsan tsan debug-stl)
set(KINEIN_PROFILE "dev" CACHE STRING "Build profile: ${KINEIN_PROFILES}")
set_property(CACHE KINEIN_PROFILE PROPERTY STRINGS ${KINEIN_PROFILES})
if(NOT KINEIN_PROFILE IN_LIST KINEIN_PROFILES)
    message(FATAL_ERROR "KINEIN_PROFILE='${KINEIN_PROFILE}' does not exist. Use one of: ${KINEIN_PROFILES}")
endif()

# C++23 without extensions. 26 exists only for the probe job (docs/engineering/cpp23-rules.md).
set(KINEIN_CXX_STANDARD "23" CACHE STRING "Language standard: 23; 26 only in the probe")
set_property(CACHE KINEIN_CXX_STANDARD PROPERTY STRINGS 23 26)
if(NOT KINEIN_CXX_STANDARD MATCHES "^(23|26)$")
    message(FATAL_ERROR "KINEIN_CXX_STANDARD='${KINEIN_CXX_STANDARD}': use 23 or 26")
endif()
set(CMAKE_CXX_STANDARD ${KINEIN_CXX_STANDARD})
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Minimum versions, the maintainer's decision of 2026-11-22: GCC 14 and Clang 20,
# the floor the CI tests on ubuntu-26.04. Before Clang 19, Clang's __cpp_concepts
# was 201907L, and libstdc++ only declares std::expected with 202002L (LLVM pull request
# 87998): with Clang 18 the core does not compile. Below the floor,
# configure fails.
if(CMAKE_CXX_COMPILER_ID STREQUAL "GNU" AND CMAKE_CXX_COMPILER_VERSION VERSION_LESS 14)
    message(FATAL_ERROR "GCC ${CMAKE_CXX_COMPILER_VERSION}: the core requires GCC 14 or newer")
elseif(CMAKE_CXX_COMPILER_ID STREQUAL "Clang" AND CMAKE_CXX_COMPILER_VERSION VERSION_LESS 20)
    message(FATAL_ERROR "Clang ${CMAKE_CXX_COMPILER_VERSION}: the core requires Clang 20 or newer")
elseif(NOT CMAKE_CXX_COMPILER_ID MATCHES "^(GNU|Clang|MSVC)$")
    message(FATAL_ERROR "Compiler ${CMAKE_CXX_COMPILER_ID} has no defined profile")
endif()

foreach(config DEBUG RELEASE RELWITHDEBINFO MINSIZEREL)
    set(CMAKE_CXX_FLAGS_${config} "")
endforeach()

# PIE everywhere; CMake adds -pie when it links executables (policy CMP0083).
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
include(CheckPIESupported)
check_pie_supported(LANGUAGES CXX)

include(KineinWarnings)
include(KineinHardening)
include(KineinSanitizers)

# Optimization and debug information per profile. FORTIFY requires -O1 or higher (OpenSSF
# Compiler Options Hardening Guide for C and C++, Table 2).
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

# kind: LIBRARY (core library), EXECUTABLE (shipped process), TEST, or
# THIRD_PARTY (external code built along with the core, such as GoogleTest).
#
# THIRD_PARTY gets everything that changes the ABI or the instrumentation
# (optimization, standard library mode, sanitizers, protections) and none of the
# warnings nor clang-tidy: the code is not ours, and -Werror on it would only
# break the build on another compiler version.
function(kinein_apply_profile target kind)
    if(NOT kind MATCHES "^(LIBRARY|EXECUTABLE|TEST|THIRD_PARTY)$")
        message(FATAL_ERROR
            "kinein_apply_profile(${target} ${kind}): use LIBRARY, EXECUTABLE, TEST or THIRD_PARTY")
    endif()
    kinein_apply_optimization("${target}")
    if(kind STREQUAL "THIRD_PARTY")
        # A warning in external code has no possible action here. Measured on 2026-11-22 with
        # Clang 23: -Wcharacter-conversion, on by default, fires in gtest-printers.h of
        # GoogleTest 1.17.0. The code that uses the framework gets its headers as -isystem.
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

message(STATUS "Kinein core: profile ${KINEIN_PROFILE}, C++${KINEIN_CXX_STANDARD}, "
               "${CMAKE_CXX_COMPILER_ID} ${CMAKE_CXX_COMPILER_VERSION}")
