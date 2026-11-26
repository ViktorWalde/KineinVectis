# Hardening flags of the core.
#
# Documented in: docs/engineering/build-profiles-and-hardening.md
#
# The flags are the whole TL;DR plus the conditional rows "for x86_64", "for aarch64" and
# "for production code" (OpenSSF Compiler Options Hardening Guide for C and C++, version of
# 2026-08-20).
# - _FORTIFY_SOURCE requires -O1 or higher (the same guide, Table 2) and the sanitizers do not
#   support it (AddressSanitizer documentation, FAQ): it stays only in the dev and production
#   profiles. The others remove it with -U, because some distributions enable it by default.
# - _GLIBCXX_DEBUG changes the size of the containers (libstdc++ manual, Debug Mode): only in
#   the debug-stl profile, with everything built together.
include_guard(GLOBAL)

function(kinein_apply_hardening target kind)
    if(MSVC)
        # Not verified: the reference machine has no MSVC.
        target_compile_options("${target}" PRIVATE /sdl /guard:cf)
        if(kind MATCHES "^(EXECUTABLE|TEST)$")
            target_link_options("${target}" PRIVATE /guard:cf /CETCOMPAT)
        endif()
        return()
    endif()

    target_compile_options("${target}" PRIVATE
        -fstack-protector-strong
        -fstack-clash-protection
        -fstrict-flex-arrays=3
    )

    if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(x86_64|AMD64|amd64)$")
        target_compile_options("${target}" PRIVATE -fcf-protection=full -fzero-call-used-regs=used-gpr)
    elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "^(aarch64|arm64)$")
        target_compile_options("${target}" PRIVATE -mbranch-protection=standard -fzero-call-used-regs=used-gpr)
    endif()

    # Standard libraries: precondition checks in every profile.
    if(KINEIN_PROFILE STREQUAL "debug-stl")
        target_compile_definitions("${target}" PRIVATE
            _GLIBCXX_DEBUG
            _GLIBCXX_DEBUG_PEDANTIC
            _LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_DEBUG
        )
    else()
        target_compile_definitions("${target}" PRIVATE
            _GLIBCXX_ASSERTIONS
            _LIBCPP_HARDENING_MODE=_LIBCPP_HARDENING_MODE_FAST
        )
    endif()

    if(KINEIN_PROFILE MATCHES "^(dev|production)$")
        target_compile_options("${target}" PRIVATE -U_FORTIFY_SOURCE -D_FORTIFY_SOURCE=3)
        # "for production code" in the guide. dev runs the same semantics as what
        # ships; the sanitizer profiles go without them, so that UBSan sees the
        # UB these flags make defined.
        target_compile_options("${target}" PRIVATE
            -fno-delete-null-pointer-checks
            -fno-strict-overflow
            -fno-strict-aliasing
            -ftrivial-auto-var-init=zero
        )
    else()
        target_compile_options("${target}" PRIVATE -U_FORTIFY_SOURCE)
    endif()

    # Libraries, ours or external, have no link step of their own.
    if(NOT kind MATCHES "^(EXECUTABLE|TEST)$" OR NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
        return()
    endif()

    # Link options of the shipped process and of the tests, the same for both.
    target_link_options("${target}" PRIVATE
        -Wl,-z,nodlopen
        -Wl,-z,noexecstack
        -Wl,-z,relro
        -Wl,-z,now
        -Wl,--as-needed
        -Wl,--no-copy-dt-needed-entries
    )
endfunction()
