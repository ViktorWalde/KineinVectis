# GoogleTest for the tests of the C++ core.
#
# Documented in: docs/engineering/test-strategy.md
#
# GoogleTest 1.17.0, BSD-3-Clause license: the maintainer's decision of 2026-11-22 (P06).
#
# Built here, with the profile, instead of the system's copy:
# - in the debug-stl profile, _GLIBCXX_DEBUG changes the ABI of the containers
#   (libstdc++ manual, Debug Mode), and the framework library must be in the same
#   mode as the test. The unit.TestFramework.* test checks this at link time;
# - under the sanitizers, the framework code is instrumented too.
#
# The tarball has a pinned hash. Without network access, point to a local copy:
#   -DFETCHCONTENT_SOURCE_DIR_GOOGLETEST=/path/to/googletest-1.17.0
include_guard(GLOBAL)

include(FetchContent)

set(INSTALL_GTEST OFF CACHE BOOL "The core does not install GoogleTest" FORCE)
set(BUILD_GMOCK ON CACHE BOOL "gMock for the port test doubles" FORCE)

FetchContent_Declare(googletest
    URL https://github.com/google/googletest/releases/download/v1.17.0/googletest-1.17.0.tar.gz
    URL_HASH SHA256=65fab701d9829d38cb77c14acdc431d2108bfdbf8979e40eb8ae567edf10b27c
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    # The headers come in as -isystem: the framework's warnings are not ours.
    SYSTEM
)
FetchContent_MakeAvailable(googletest)

foreach(target IN ITEMS gtest gtest_main gmock gmock_main)
    kinein_apply_profile("${target}" THIRD_PARTY)
endforeach()

include(GoogleTest)

# kinein_add_unit_test(<name> SOURCES <files...> [LIBRARIES <targets...>])
#
# One executable per module, with the test profile, and each TEST becomes a CTest
# test with the "unit." prefix. Discovery runs at test time (PRE_TEST), not at
# build time: the binary only runs under the sanitizer environment.
function(kinein_add_unit_test name)
    cmake_parse_arguments(PARSE_ARGV 1 arg "" "" "SOURCES;LIBRARIES")
    if(NOT arg_SOURCES)
        message(FATAL_ERROR "kinein_add_unit_test(${name}): SOURCES is missing")
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
