# Tests of the build contract: the flags the profile promises reach the compiler and the binary.
#
# Documented in: docs/engineering/build-profiles-and-hardening.md
#
# build.contract checks the compiler side; build.elf_hardening checks the linked binary.
add_executable(kinein_build_contract_test build_contract_test.cpp)
kinein_apply_profile(kinein_build_contract_test TEST)

string(MAKE_C_IDENTIFIER "${KINEIN_PROFILE}" profile_id)
string(TOUPPER "${profile_id}" profile_id)
target_compile_definitions(kinein_build_contract_test PRIVATE
    KINEIN_EXPECTED_PROFILE_${profile_id}=1
    KINEIN_EXPECTED_CXX_STANDARD=${KINEIN_CXX_STANDARD}
)

add_test(NAME build.contract COMMAND kinein_build_contract_test)
kinein_sanitizer_test_environment(build.contract)

if(CMAKE_SYSTEM_NAME STREQUAL "Linux")
    find_program(KINEIN_READELF NAMES readelf REQUIRED)
    add_test(NAME build.elf_hardening
        COMMAND "${CMAKE_COMMAND}"
            "-DBINARY=$<TARGET_FILE:kinein_build_contract_test>"
            "-DREADELF=${KINEIN_READELF}"
            -P "${PROJECT_SOURCE_DIR}/cmake/CheckElfHardening.cmake")
endif()
