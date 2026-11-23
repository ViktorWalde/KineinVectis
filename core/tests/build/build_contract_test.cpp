// Contract of the build profile, checked by the compiler itself.
//
// Documented in: docs/engenharia/perfis-de-compilacao-e-hardening.md
//
// CMake declares the profile (KINEIN_EXPECTED_PROFILE_*) and the language standard
// (KINEIN_EXPECTED_CXX_STANDARD). Each #error or static_assert that fires here is a build defect,
// not a code defect: a promised flag that did not arrive.

// <cstdlib> also brings glibc's <features.h> and libstdc++'s <bits/c++config.h>.
#include <cstdlib>

#if !defined(KINEIN_EXPECTED_CXX_STANDARD)
#error "KINEIN_EXPECTED_CXX_STANDARD was not defined by CMake"
#endif

namespace {

#if defined(_MSVC_LANG)
constexpr long language_version = _MSVC_LANG;
#else
constexpr long language_version = __cplusplus;
#endif

#if KINEIN_EXPECTED_CXX_STANDARD == 23
static_assert(language_version == 202302L, "the profile promises C++23");
#elif KINEIN_EXPECTED_CXX_STANDARD == 26
static_assert(language_version > 202302L, "the probe promises C++26");
#else
#error "KINEIN_EXPECTED_CXX_STANDARD must be 23 or 26"
#endif

} // namespace

// No GNU extensions: -std=c++23, not -std=gnu++23.
#if (defined(__GNUC__) || defined(__clang__)) && !defined(__STRICT_ANSI__)
#error "GNU extensions enabled: CMAKE_CXX_EXTENSIONS OFF is missing"
#endif

#if defined(__GNUC__) || defined(__clang__)

#if !defined(__PIE__)
#error "no -fPIE"
#endif

#if !defined(__SSP_STRONG__)
#error "no -fstack-protector-strong"
#endif

#if defined(__x86_64__) && !defined(__CET__)
#error "no -fcf-protection on x86_64"
#endif

// Standard library precondition checks: in every profile.
#if defined(__GLIBCXX__) && !defined(_GLIBCXX_ASSERTIONS) && !defined(_GLIBCXX_DEBUG)
#error "libstdc++ without _GLIBCXX_ASSERTIONS or _GLIBCXX_DEBUG"
#endif

// --- Profiles with the semantics of what ships: effective FORTIFY level 3. ---
#if defined(KINEIN_EXPECTED_PROFILE_DEV) || defined(KINEIN_EXPECTED_PROFILE_PRODUCTION)
#if !defined(_FORTIFY_SOURCE)
#error "dev/production without _FORTIFY_SOURCE"
#endif
#if defined(__GLIBC__)
// __USE_FORTIFY_LEVEL is the level glibc actually applied after seeing the
// optimization and the compiler; _FORTIFY_SOURCE alone is only the request.
#if !defined(__USE_FORTIFY_LEVEL)
#error "glibc without __USE_FORTIFY_LEVEL"
#elif __USE_FORTIFY_LEVEL != 3
#error "glibc did not apply FORTIFY level 3"
#endif
#endif
#if defined(_GLIBCXX_DEBUG)
#error "_GLIBCXX_DEBUG outside the debug-stl profile changes the size of the containers"
#endif
#endif

// --- Sanitizer profiles: no FORTIFY, which the sanitizers do not support. ---
#if defined(KINEIN_EXPECTED_PROFILE_ASAN_UBSAN) || defined(KINEIN_EXPECTED_PROFILE_TSAN)
#if defined(_FORTIFY_SOURCE)
#error "sanitizer with _FORTIFY_SOURCE enabled"
#endif
#endif

#if defined(KINEIN_EXPECTED_PROFILE_ASAN_UBSAN)
#if defined(__has_feature)
#if !__has_feature(address_sanitizer) && !defined(__SANITIZE_ADDRESS__)
#error "asan-ubsan profile without AddressSanitizer"
#endif
#elif !defined(__SANITIZE_ADDRESS__)
#error "asan-ubsan profile without AddressSanitizer"
#endif
#endif

#if defined(KINEIN_EXPECTED_PROFILE_TSAN)
#if defined(__has_feature)
#if !__has_feature(thread_sanitizer) && !defined(__SANITIZE_THREAD__)
#error "tsan profile without ThreadSanitizer"
#endif
#elif !defined(__SANITIZE_THREAD__)
#error "tsan profile without ThreadSanitizer"
#endif
#endif

// --- Standard library debug profile. ---
#if defined(KINEIN_EXPECTED_PROFILE_DEBUG_STL) && defined(__GLIBCXX__) && !defined(_GLIBCXX_DEBUG)
#error "debug-stl profile without _GLIBCXX_DEBUG"
#endif

#endif // GCC or Clang

int main()
{
    return EXIT_SUCCESS;
}
