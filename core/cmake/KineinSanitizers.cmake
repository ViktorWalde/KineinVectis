# Sanitizers of the C++ core.
#
# Documented in: docs/engineering/build-profiles-and-hardening.md
#
# ASan and UBSan together; TSan in a separate build, because the compiler rejects
# -fsanitize=thread together with -fsanitize=address. A sanitizer error fails the
# test: no recovering and carrying on.
include_guard(GLOBAL)

if(MSVC AND KINEIN_PROFILE STREQUAL "tsan")
    message(FATAL_ERROR "MSVC has no ThreadSanitizer: run the tsan profile on Linux")
endif()

function(kinein_apply_sanitizers target)
    if(KINEIN_PROFILE STREQUAL "asan-ubsan")
        if(MSVC)
            # MSVC has ASan and no UBSan, as cmake/KineinStrictOptions.cmake states. Not
            # verified: the reference machine has no MSVC.
            target_compile_options("${target}" PRIVATE /fsanitize=address)
            return()
        endif()
        set(flags -fsanitize=address,undefined -fno-sanitize-recover=all)
    elseif(KINEIN_PROFILE STREQUAL "tsan")
        set(flags -fsanitize=thread)
    else()
        return()
    endif()
    target_compile_options("${target}" PRIVATE ${flags})
    target_link_options("${target}" PRIVATE ${flags})
endfunction()

# Test runtime environment of the current profile, as a list of VAR=value.
if(KINEIN_PROFILE STREQUAL "asan-ubsan")
    set(KINEIN_SANITIZER_ENVIRONMENT
        "ASAN_OPTIONS=detect_leaks=1:strict_string_checks=1:detect_stack_use_after_return=1:check_initialization_order=1:strict_init_order=1:halt_on_error=1"
        "UBSAN_OPTIONS=print_stacktrace=1:halt_on_error=1"
    )
elseif(KINEIN_PROFILE STREQUAL "tsan")
    set(KINEIN_SANITIZER_ENVIRONMENT "TSAN_OPTIONS=halt_on_error=1:second_deadlock_stack=1")
else()
    set(KINEIN_SANITIZER_ENVIRONMENT "")
endif()

function(kinein_sanitizer_test_environment test_name)
    if(KINEIN_SANITIZER_ENVIRONMENT)
        set_property(TEST "${test_name}" APPEND PROPERTY ENVIRONMENT ${KINEIN_SANITIZER_ENVIRONMENT})
    endif()
endfunction()
