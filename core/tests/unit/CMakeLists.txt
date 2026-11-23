# Testes de unidade de core/lib, com GoogleTest (docs/engenharia/estrategia-de-testes.md).
# Um executavel por modulo, por kinein_add_unit_test, sem laco de eventos e sem
# processo de verdade.
include(KineinTesting)

kinein_add_unit_test(kinein_framework_test SOURCES framework_test.cpp)
