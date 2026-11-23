# Endurecimento do core (docs/engenharia/perfis-de-compilacao-e-hardening.md).
#
# Fonte: OpenSSF Compiler Options Hardening Guide for C and C++, versao de
# 2026-08-20: o TL;DR inteiro, mais as linhas condicionais "for x86_64",
# "for aarch64" e "for production code".
# - O _FORTIFY_SOURCE exige -O1 ou mais (Tabela 2 do guia) e os sanitizers nao o
#   suportam (FAQ do AddressSanitizer): fica so nos perfis dev e production.
#   Nos outros sai com -U, porque ha distribuicao que o liga por padrao.
# - O _GLIBCXX_DEBUG muda o tamanho dos containers (manual da libstdc++): so no
#   perfil debug-stl, com tudo compilado junto.
include_guard(GLOBAL)

function(kinein_apply_hardening target kind)
    if(MSVC)
        # Herdado do KineinStrictOptions.cmake; nao conferido nesta maquina.
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

    # Bibliotecas padrao: verificacao de precondicao em todos os perfis.
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
        # "for production code" no guia. O dev roda a mesma semantica do que se
        # entrega; os perfis de sanitizer ficam sem ela, para o UBSan ver o UB
        # que estas flags tornam definido.
        target_compile_options("${target}" PRIVATE
            -fno-delete-null-pointer-checks
            -fno-strict-overflow
            -fno-strict-aliasing
            -ftrivial-auto-var-init=zero
        )
    else()
        target_compile_options("${target}" PRIVATE -U_FORTIFY_SOURCE)
    endif()

    # Bibliotecas, nossas ou de fora, nao tem link proprio.
    if(NOT kind MATCHES "^(EXECUTABLE|TEST)$" OR NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
        return()
    endif()

    # Link do processo entregue e dos testes, com o mesmo formato.
    target_link_options("${target}" PRIVATE
        -Wl,-z,nodlopen
        -Wl,-z,noexecstack
        -Wl,-z,relro
        -Wl,-z,now
        -Wl,--as-needed
        -Wl,--no-copy-dt-needed-entries
    )
endfunction()
