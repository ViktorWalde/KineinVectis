# Unit tests of core/lib, with GoogleTest.
#
# Documented in: docs/engenharia/estrategia-de-testes.md
#
# One executable per module, through kinein_add_unit_test, with no event loop and no real
# process.
include(KineinTesting)

kinein_add_unit_test(kinein_framework_test SOURCES framework_test.cpp)
