# Core library target: domain, rules and algorithms, with no Qt and no event loop.
#
# Documented in: docs/architecture/boundaries-and-responsibilities.md
#
# The code rules are in docs/engineering/cpp23-rules.md. No code yet: the target exists to fix
# its name and dependencies. With the first .cpp it becomes STATIC and gets
#     kinein_apply_profile(kinein_core_lib LIBRARY)
# It never gets AUTOMOC nor a Qt link; the dev preset does not even look for Qt.
add_library(kinein_core_lib INTERFACE)
add_library(kinein::core_lib ALIAS kinein_core_lib)
target_compile_features(kinein_core_lib INTERFACE cxx_std_${KINEIN_CXX_STANDARD})
