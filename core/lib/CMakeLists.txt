# Biblioteca do core: dominio, regras e algoritmos, sem Qt e sem laco de eventos.
# Regras: docs/arquitetura/fronteiras-e-responsabilidades.md e
# docs/engenharia/regras-de-cpp23.md.
#
# Ainda sem codigo: o alvo existe para fixar o nome e as dependencias. Com o
# primeiro .cpp ele passa a STATIC e recebe
#     kinein_apply_profile(kinein_core_lib LIBRARY)
# Nunca recebe AUTOMOC nem link com Qt; o preset dev nem procura o Qt.
add_library(kinein_core_lib INTERFACE)
add_library(kinein::core_lib ALIAS kinein_core_lib)
target_compile_features(kinein_core_lib INTERFACE cxx_std_${KINEIN_CXX_STANDARD})
