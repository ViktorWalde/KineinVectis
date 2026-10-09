include(CheckCXXCompilerFlag)

option(KINEIN_WARNINGS_AS_ERRORS "Treat C++ compiler warnings as errors" ON)
option(KINEIN_ENABLE_SANITIZERS "Enable ASan and UBSan for debug native builds" ON)
option(KINEIN_ENABLE_HARDENING "Enable hardened release compiler and linker flags" ON)

# O ASan do MSVC recusa o /RTC1 que o CMake poe no Debug (60 §3.2, W3b).
if(MSVC AND KINEIN_ENABLE_SANITIZERS)
    string(REPLACE "/RTC1" "" CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG}")
endif()

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
    elseif(MSVC)
        # O MSVC (DocsPublic/roadmaps/60 §3.2, W3b): o mais perto do bloco
        # acima. /W4 e' o nivel que a Microsoft recomenda para codigo novo; /sdl
        # liga as verificacoes de seguranca e faz de erro os avisos delas;
        # /guard:cf e' o par do -fcf-protection. O /permissive-, o /utf-8 (as
        # fontes tem acento em texto de tela) e o /Zc:__cplusplus vem do Qt, e
        # ficam aqui tambem para nao depender dele.
        target_compile_options(
            "${target}"
            PRIVATE
                /W4
                /sdl
                /permissive-
                /utf-8
                /Zc:__cplusplus
                /Zc:preprocessor
                /guard:cf
        )
        target_link_options("${target}" PRIVATE /guard:cf /CETCOMPAT)
        if(KINEIN_WARNINGS_AS_ERRORS)
            target_compile_options("${target}" PRIVATE /WX)
        endif()
    endif()

    if(KINEIN_ENABLE_SANITIZERS AND CMAKE_BUILD_TYPE STREQUAL "Debug")
        if(MSVC)
            # O MSVC tem o ASan e nao tem o UBSan. Antes de 2026-10-09 a flag
            # do Clang chegava ao `cl`, que a ignorava com um aviso (D9002), e o
            # debug do Windows rodava sem sanitizer nenhum sem ninguem notar. O
            # ASan nao convive com o /RTC1 do Debug padrao: ele sai no topo
            # deste arquivo.
            target_compile_options("${target}" PRIVATE /fsanitize=address)
        else()
            target_compile_options("${target}" PRIVATE -fsanitize=address,undefined)
            target_link_options("${target}" PRIVATE -fsanitize=address,undefined)
        endif()
    endif()

    if(KINEIN_ENABLE_HARDENING AND CMAKE_BUILD_TYPE STREQUAL "Release")
        set_property(TARGET "${target}" PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
        if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
            target_compile_options("${target}" PRIVATE -D_FORTIFY_SOURCE=3)
            target_link_options("${target}" PRIVATE -Wl,-z,relro -Wl,-z,now -Wl,--as-needed)
        endif()
    endif()
endfunction()
