// Contrato do perfil de compilacao, conferido pelo proprio compilador.
//
// O CMake declara o perfil (KINEIN_EXPECTED_PROFILE_*) e o padrao da linguagem
// (KINEIN_EXPECTED_CXX_STANDARD). Cada #error ou static_assert que dispara aqui
// e' um defeito do build, nao do codigo: uma flag prometida que nao chegou.
// Tabela dos perfis: docs/engenharia/perfis-de-compilacao-e-hardening.md.

#include <cstdlib>  // tambem traz <features.h> da glibc e <bits/c++config.h> da libstdc++

#if !defined(KINEIN_EXPECTED_CXX_STANDARD)
#error "KINEIN_EXPECTED_CXX_STANDARD nao foi definido pelo CMake"
#endif

namespace {

#if defined(_MSVC_LANG)
constexpr long language_version = _MSVC_LANG;
#else
constexpr long language_version = __cplusplus;
#endif

#if KINEIN_EXPECTED_CXX_STANDARD == 23
static_assert(language_version == 202302L, "o perfil promete C++23");
#elif KINEIN_EXPECTED_CXX_STANDARD == 26
static_assert(language_version > 202302L, "a sondagem promete C++26");
#else
#error "KINEIN_EXPECTED_CXX_STANDARD tem de ser 23 ou 26"
#endif

}  // namespace

// Sem extensoes GNU: -std=c++23, e nao -std=gnu++23.
#if (defined(__GNUC__) || defined(__clang__)) && !defined(__STRICT_ANSI__)
#error "extensoes GNU ligadas: falta CMAKE_CXX_EXTENSIONS OFF"
#endif

#if defined(__GNUC__) || defined(__clang__)

#if !defined(__PIE__)
#error "sem -fPIE"
#endif

#if !defined(__SSP_STRONG__)
#error "sem -fstack-protector-strong"
#endif

#if defined(__x86_64__) && !defined(__CET__)
#error "sem -fcf-protection no x86_64"
#endif

// Verificacao de precondicao da biblioteca padrao: em todos os perfis.
#if defined(__GLIBCXX__) && !defined(_GLIBCXX_ASSERTIONS) && !defined(_GLIBCXX_DEBUG)
#error "libstdc++ sem _GLIBCXX_ASSERTIONS nem _GLIBCXX_DEBUG"
#endif

// --- Perfis com a semantica do que se entrega: FORTIFY nivel 3 efetivo. ---
#if defined(KINEIN_EXPECTED_PROFILE_DEV) || defined(KINEIN_EXPECTED_PROFILE_PRODUCTION)
#if !defined(_FORTIFY_SOURCE)
#error "dev/production sem _FORTIFY_SOURCE"
#endif
#if defined(__GLIBC__)
// __USE_FORTIFY_LEVEL e' o nivel que a glibc de fato aplicou depois de ver a
// otimizacao e o compilador; _FORTIFY_SOURCE sozinho e' so o pedido.
#if !defined(__USE_FORTIFY_LEVEL)
#error "glibc sem __USE_FORTIFY_LEVEL"
#elif __USE_FORTIFY_LEVEL != 3
#error "a glibc nao aplicou o FORTIFY nivel 3"
#endif
#endif
#if defined(_GLIBCXX_DEBUG)
#error "_GLIBCXX_DEBUG fora do perfil debug-stl muda o tamanho dos containers"
#endif
#endif

// --- Perfis de sanitizer: sem FORTIFY, que os sanitizers nao suportam. ---
#if defined(KINEIN_EXPECTED_PROFILE_ASAN_UBSAN) || defined(KINEIN_EXPECTED_PROFILE_TSAN)
#if defined(_FORTIFY_SOURCE)
#error "sanitizer com _FORTIFY_SOURCE ligado"
#endif
#endif

#if defined(KINEIN_EXPECTED_PROFILE_ASAN_UBSAN)
#if defined(__has_feature)
#if !__has_feature(address_sanitizer) && !defined(__SANITIZE_ADDRESS__)
#error "perfil asan-ubsan sem AddressSanitizer"
#endif
#elif !defined(__SANITIZE_ADDRESS__)
#error "perfil asan-ubsan sem AddressSanitizer"
#endif
#endif

#if defined(KINEIN_EXPECTED_PROFILE_TSAN)
#if defined(__has_feature)
#if !__has_feature(thread_sanitizer) && !defined(__SANITIZE_THREAD__)
#error "perfil tsan sem ThreadSanitizer"
#endif
#elif !defined(__SANITIZE_THREAD__)
#error "perfil tsan sem ThreadSanitizer"
#endif
#endif

// --- Perfil de teste da biblioteca padrao. ---
#if defined(KINEIN_EXPECTED_PROFILE_DEBUG_STL) && defined(__GLIBCXX__) && !defined(_GLIBCXX_DEBUG)
#error "perfil debug-stl sem _GLIBCXX_DEBUG"
#endif

#endif  // GCC ou Clang

int main() {
    return EXIT_SUCCESS;
}
