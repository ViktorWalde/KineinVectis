# Confere no binario o que o KineinHardening.cmake promete no link.
# Uso: cmake -DBINARY=<executavel> -DREADELF=<readelf> -P CheckElfHardening.cmake
#
# O teste le o ELF que o build produziu, e nao as flags do CMake: verde aqui quer
# dizer que a protecao chegou ao arquivo (licao do INF-31, docs/licoes-da-0.3.md).

if(NOT DEFINED BINARY OR NOT EXISTS "${BINARY}")
    message(FATAL_ERROR "BINARY nao existe: '${BINARY}'")
endif()
if(NOT DEFINED READELF)
    message(FATAL_ERROR "READELF nao definido")
endif()

execute_process(COMMAND "${READELF}" -W --dynamic "${BINARY}"
    OUTPUT_VARIABLE dynamic RESULT_VARIABLE dynamic_rc ERROR_VARIABLE dynamic_err)
execute_process(COMMAND "${READELF}" -W --program-headers "${BINARY}"
    OUTPUT_VARIABLE segments RESULT_VARIABLE segments_rc ERROR_VARIABLE segments_err)
if(NOT dynamic_rc EQUAL 0 OR NOT segments_rc EQUAL 0)
    message(FATAL_ERROR "readelf falhou: ${dynamic_err}${segments_err}")
endif()

set(failures "")

# -Wl,-z,now: BIND_NOW, ou NOW no FLAGS_1.
if(NOT dynamic MATCHES "\\(FLAGS\\)[^\n]*BIND_NOW" AND NOT dynamic MATCHES "\\(FLAGS_1\\)[^\n]* NOW")
    list(APPEND failures "sem BIND_NOW (-Wl,-z,now)")
endif()
# -fPIE -pie
if(NOT dynamic MATCHES "\\(FLAGS_1\\)[^\n]* PIE")
    list(APPEND failures "sem PIE (-fPIE -pie)")
endif()
# -Wl,-z,nodlopen marca so biblioteca compartilhada: medido em 2026-11-22 com o
# GNU ld 2.47, o executavel PIE sai sem NOOPEN. O core nao gera biblioteca
# compartilhada; quando gerar, a conferencia do NOOPEN entra aqui para ela.
# -Wl,-z,relro
if(NOT segments MATCHES "GNU_RELRO")
    list(APPEND failures "sem GNU_RELRO (-Wl,-z,relro)")
endif()
# -Wl,-z,noexecstack: o segmento GNU_STACK existe e nao tem E.
string(REGEX MATCH "GNU_STACK[^\n]*" stack_line "${segments}")
if(stack_line STREQUAL "")
    list(APPEND failures "sem segmento GNU_STACK")
elseif(stack_line MATCHES " RWE( |$)")
    list(APPEND failures "pilha executavel (-Wl,-z,noexecstack)")
endif()

if(failures)
    list(JOIN failures "; " text)
    message(FATAL_ERROR "${BINARY}: ${text}")
endif()
message(STATUS "ELF endurecido: ${BINARY}")
