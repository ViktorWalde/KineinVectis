include(CheckCXXCompilerFlag)

option(KERNWERK_WARNINGS_AS_ERRORS "Treat C++ compiler warnings as errors" ON)
option(KERNWERK_ENABLE_SANITIZERS "Enable ASan and UBSan for debug native builds" ON)
option(KERNWERK_ENABLE_HARDENING "Enable hardened release compiler and linker flags" ON)

function(kernwerk_add_supported_cxx_option target option)
    string(MAKE_C_IDENTIFIER "KERNWERK_SUPPORTS_${option}" cache_key)
    check_cxx_compiler_flag("${option}" "${cache_key}")
    if(${cache_key})
        target_compile_options("${target}" PRIVATE "${option}")
    endif()
endfunction()

function(kernwerk_enable_strict_compiler_options target)
    target_compile_features("${target}" PRIVATE cxx_std_23)

    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
        target_compile_options(
            "${target}"
            PRIVATE
                -Wall
                -Wextra
                -Wpedantic
                -Wconversion
                -Wsign-conversion
                -Wshadow
                -Wnon-virtual-dtor
                -Wold-style-cast
                -Wcast-align
                -Wunused
                -Woverloaded-virtual
                -Wnull-dereference
                -Wdouble-promotion
                -Wformat=2
                -Wimplicit-fallthrough
                -Wundef
                -fstack-protector-strong
                -fno-omit-frame-pointer
                -D_GLIBCXX_ASSERTIONS
        )

        if(KERNWERK_WARNINGS_AS_ERRORS)
            target_compile_options("${target}" PRIVATE -Werror)
        endif()

        kernwerk_add_supported_cxx_option("${target}" -Wcast-qual)
        kernwerk_add_supported_cxx_option("${target}" -Wduplicated-branches)
        kernwerk_add_supported_cxx_option("${target}" -Wduplicated-cond)
        kernwerk_add_supported_cxx_option("${target}" -Wextra-semi)
        kernwerk_add_supported_cxx_option("${target}" -Wfloat-equal)
        kernwerk_add_supported_cxx_option("${target}" -Wimplicit-float-conversion)
        kernwerk_add_supported_cxx_option("${target}" -Wmissing-declarations)
        kernwerk_add_supported_cxx_option("${target}" -Wredundant-decls)
        kernwerk_add_supported_cxx_option("${target}" -Wstrict-overflow=5)
        kernwerk_add_supported_cxx_option("${target}" -Wswitch-enum)
        kernwerk_add_supported_cxx_option("${target}" -Wzero-as-null-pointer-constant)
        kernwerk_add_supported_cxx_option("${target}" -fstrict-flex-arrays=3)
        kernwerk_add_supported_cxx_option("${target}" -Wcomma)
        kernwerk_add_supported_cxx_option("${target}" -Wctad-maybe-unsupported)
        kernwerk_add_supported_cxx_option("${target}" -Wdate-time)
        kernwerk_add_supported_cxx_option("${target}" -Wdeprecated)
        kernwerk_add_supported_cxx_option("${target}" -Wheader-hygiene)
        kernwerk_add_supported_cxx_option("${target}" -Wlogical-op)
        kernwerk_add_supported_cxx_option("${target}" -Wloop-analysis)
        kernwerk_add_supported_cxx_option("${target}" -Wmissing-noreturn)
        kernwerk_add_supported_cxx_option("${target}" -Wshift-overflow)
        kernwerk_add_supported_cxx_option("${target}" -Wsuggest-override)
        kernwerk_add_supported_cxx_option("${target}" -Wtrampolines)
        kernwerk_add_supported_cxx_option("${target}" -Wunreachable-code)
        kernwerk_add_supported_cxx_option("${target}" -Wuseless-cast)
        kernwerk_add_supported_cxx_option("${target}" -fcf-protection=full)
    endif()

    if(KERNWERK_ENABLE_SANITIZERS AND CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options("${target}" PRIVATE -fsanitize=address,undefined)
        target_link_options("${target}" PRIVATE -fsanitize=address,undefined)
    endif()

    if(KERNWERK_ENABLE_HARDENING AND CMAKE_BUILD_TYPE STREQUAL "Release")
        set_property(TARGET "${target}" PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
        if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
            target_compile_options("${target}" PRIVATE -D_FORTIFY_SOURCE=3)
            target_link_options("${target}" PRIVATE -Wl,-z,relro -Wl,-z,now -Wl,--as-needed)
        endif()
    endif()
endfunction()
