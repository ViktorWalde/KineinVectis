include(CheckCXXCompilerFlag)

option(KINEIN_WARNINGS_AS_ERRORS "Treat C++ compiler warnings as errors" ON)
option(KINEIN_ENABLE_SANITIZERS "Enable ASan and UBSan for debug native builds" ON)
option(KINEIN_ENABLE_HARDENING "Enable hardened release compiler and linker flags" ON)

function(kinein_add_supported_cxx_option target option)
    string(MAKE_C_IDENTIFIER "KINEIN_SUPPORTS_${option}" cache_key)
    check_cxx_compiler_flag("${option}" "${cache_key}")
    if(${cache_key})
        target_compile_options("${target}" PRIVATE "${option}")
    endif()
endfunction()

function(kinein_enable_strict_compiler_options target)
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

        if(KINEIN_WARNINGS_AS_ERRORS)
            target_compile_options("${target}" PRIVATE -Werror)
        endif()

        kinein_add_supported_cxx_option("${target}" -Wcast-qual)
        kinein_add_supported_cxx_option("${target}" -Wduplicated-branches)
        kinein_add_supported_cxx_option("${target}" -Wduplicated-cond)
        kinein_add_supported_cxx_option("${target}" -Wextra-semi)
        kinein_add_supported_cxx_option("${target}" -Wfloat-equal)
        kinein_add_supported_cxx_option("${target}" -Wimplicit-float-conversion)
        kinein_add_supported_cxx_option("${target}" -Wmissing-declarations)
        kinein_add_supported_cxx_option("${target}" -Wredundant-decls)
        kinein_add_supported_cxx_option("${target}" -Wstrict-overflow=5)
        kinein_add_supported_cxx_option("${target}" -Wswitch-enum)
        kinein_add_supported_cxx_option("${target}" -Wzero-as-null-pointer-constant)
        kinein_add_supported_cxx_option("${target}" -fstrict-flex-arrays=3)
        kinein_add_supported_cxx_option("${target}" -Wcomma)
        kinein_add_supported_cxx_option("${target}" -Wctad-maybe-unsupported)
        kinein_add_supported_cxx_option("${target}" -Wdate-time)
        kinein_add_supported_cxx_option("${target}" -Wdeprecated)
        kinein_add_supported_cxx_option("${target}" -Wheader-hygiene)
        kinein_add_supported_cxx_option("${target}" -Wlogical-op)
        kinein_add_supported_cxx_option("${target}" -Wloop-analysis)
        kinein_add_supported_cxx_option("${target}" -Wmissing-noreturn)
        kinein_add_supported_cxx_option("${target}" -Wshift-overflow)
        kinein_add_supported_cxx_option("${target}" -Wsuggest-override)
        kinein_add_supported_cxx_option("${target}" -Wtrampolines)
        kinein_add_supported_cxx_option("${target}" -Wunreachable-code)
        kinein_add_supported_cxx_option("${target}" -Wuseless-cast)
        kinein_add_supported_cxx_option("${target}" -fcf-protection=full)
    endif()

    if(KINEIN_ENABLE_SANITIZERS AND CMAKE_BUILD_TYPE STREQUAL "Debug")
        target_compile_options("${target}" PRIVATE -fsanitize=address,undefined)
        target_link_options("${target}" PRIVATE -fsanitize=address,undefined)
    endif()

    if(KINEIN_ENABLE_HARDENING AND CMAKE_BUILD_TYPE STREQUAL "Release")
        set_property(TARGET "${target}" PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
        if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
            target_compile_options("${target}" PRIVATE -D_FORTIFY_SOURCE=3)
            target_link_options("${target}" PRIVATE -Wl,-z,relro -Wl,-z,now -Wl,--as-needed)
        endif()
    endif()
endfunction()
