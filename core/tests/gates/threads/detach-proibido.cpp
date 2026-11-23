// Corpus do gate "thread nunca solta" (CP.26): os defeitos.
// (docs/engenharia/gates-e-medida-de-eficacia.md)
//
// Cada linha marcada com "reprova:" e' um defeito. O clang-tidy, com a
// opcao bugprone-unsafe-functions.CustomFunctions do core/.clang-tidy, tem
// de acusar exatamente essas linhas e nenhuma outra.
#include <functional>
#include <thread>

namespace corpus {

void work();

struct Worker : std::thread
{
    using std::thread::thread;
};

#define KINEIN_CORPUS_DETACH(thread_object) (thread_object).detach()

void d01() { std::thread t(work); t.detach(); } // reprova: detach de std::thread
void d02() { std::jthread t(work); t.detach(); } // reprova: detach de std::jthread
void d03(std::thread* t) { t->detach(); } // reprova: detach por ponteiro
void d04(std::thread& t) { t.detach(); } // reprova: detach por referencia
auto d05() { return &std::thread::detach; } // reprova: ponteiro para o metodo
void d06(std::thread& t) { std::invoke(&std::thread::detach, t); } // reprova: metodo passado a std::invoke
void d07(std::thread& t) { [&t] { t.detach(); }(); } // reprova: detach dentro de lambda
void d08() { Worker w(work); w.detach(); } // reprova: detach herdado por classe derivada
void d09(std::thread& t) { KINEIN_CORPUS_DETACH(t); } // reprova: detach dentro de macro
void d10() { std::thread{work}.detach(); } // reprova: detach de temporario

} // namespace corpus
