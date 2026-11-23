# Check in the binary what core/cmake/KineinHardening.cmake promises at link time.
#
# Documented in: docs/engenharia/perfis-de-compilacao-e-hardening.md
#
# Usage: cmake -DBINARY=<executable> -DREADELF=<readelf> -P CheckElfHardening.cmake
#
# The test reads the ELF the build produced, not the CMake flags: green here means that the
# protection reached the file (lesson L17 in docs/licoes-da-0.3.md).

if(NOT DEFINED BINARY OR NOT EXISTS "${BINARY}")
    message(FATAL_ERROR "BINARY does not exist: '${BINARY}'")
endif()
if(NOT DEFINED READELF)
    message(FATAL_ERROR "READELF is not defined")
endif()

execute_process(COMMAND "${READELF}" -W --dynamic "${BINARY}"
    OUTPUT_VARIABLE dynamic RESULT_VARIABLE dynamic_rc ERROR_VARIABLE dynamic_err)
execute_process(COMMAND "${READELF}" -W --program-headers "${BINARY}"
    OUTPUT_VARIABLE segments RESULT_VARIABLE segments_rc ERROR_VARIABLE segments_err)
if(NOT dynamic_rc EQUAL 0 OR NOT segments_rc EQUAL 0)
    message(FATAL_ERROR "readelf failed: ${dynamic_err}${segments_err}")
endif()

set(failures "")

# -Wl,-z,now: BIND_NOW, or NOW in FLAGS_1.
if(NOT dynamic MATCHES "\\(FLAGS\\)[^\n]*BIND_NOW" AND NOT dynamic MATCHES "\\(FLAGS_1\\)[^\n]* NOW")
    list(APPEND failures "no BIND_NOW (-Wl,-z,now)")
endif()
# -fPIE -pie
if(NOT dynamic MATCHES "\\(FLAGS_1\\)[^\n]* PIE")
    list(APPEND failures "no PIE (-fPIE -pie)")
endif()
# -Wl,-z,nodlopen only marks shared libraries. Measured on 2026-11-22 with GNU ld 2.47: a PIE
# executable comes out without NOOPEN. The core builds no
# shared library; when it does, the NOOPEN check goes here for that library.
# -Wl,-z,relro
if(NOT segments MATCHES "GNU_RELRO")
    list(APPEND failures "no GNU_RELRO (-Wl,-z,relro)")
endif()
# -Wl,-z,noexecstack: the GNU_STACK segment exists and has no E.
string(REGEX MATCH "GNU_STACK[^\n]*" stack_line "${segments}")
if(stack_line STREQUAL "")
    list(APPEND failures "no GNU_STACK segment")
elseif(stack_line MATCHES " RWE( |$)")
    list(APPEND failures "executable stack (-Wl,-z,noexecstack)")
endif()

if(failures)
    list(JOIN failures "; " text)
    message(FATAL_ERROR "${BINARY}: ${text}")
endif()
message(STATUS "ELF hardened: ${BINARY}")
