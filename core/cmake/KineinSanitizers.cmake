# Sanitizers do core (docs/engenharia/perfis-de-compilacao-e-hardening.md).
#
# ASan e UBSan juntos; TSan num build separado, porque o compilador recusa
# -fsanitize=thread junto de -fsanitize=address. Erro do sanitizer reprova o
# teste: nada de recuperar e seguir.
include_guard(GLOBAL)

if(MSVC AND KINEIN_PROFILE STREQUAL "tsan")
    message(FATAL_ERROR "O MSVC nao tem ThreadSanitizer: rode o perfil tsan no Linux")
endif()

function(kinein_apply_sanitizers target)
    if(KINEIN_PROFILE STREQUAL "asan-ubsan")
        if(MSVC)
            # O MSVC tem ASan e nao tem UBSan (KineinStrictOptions.cmake).
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

# Ambiente de execucao dos testes no perfil atual, como lista de VAR=valor.
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
