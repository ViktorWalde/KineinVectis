// Corpus of gate.detach: the planted defects of "no detached thread".
//
// Documented in: docs/engineering/gates-and-effectiveness.md
//
// The rule is C++ Core Guidelines CP.26. Each line marked with "reject:" is a defect. clang-tidy,
// with the bugprone-unsafe-functions.CustomFunctions option of core/.clang-tidy, must flag
// exactly these lines and no other.
#include <functional>
#include <thread>

namespace corpus {

void work();

struct Worker : std::thread
{
    using std::thread::thread;
};

#define KINEIN_CORPUS_DETACH(thread_object) (thread_object).detach()

void d01() { std::thread t(work); t.detach(); } // reject: detach of std::thread
void d02() { std::jthread t(work); t.detach(); } // reject: detach of std::jthread
void d03(std::thread* t) { t->detach(); } // reject: detach through a pointer
void d04(std::thread& t) { t.detach(); } // reject: detach through a reference
auto d05() { return &std::thread::detach; } // reject: pointer to the method
void d06(std::thread& t) { std::invoke(&std::thread::detach, t); } // reject: method passed to std::invoke
void d07(std::thread& t) { [&t] { t.detach(); }(); } // reject: detach inside a lambda
void d08() { Worker w(work); w.detach(); } // reject: detach inherited by a derived class
void d09(std::thread& t) { KINEIN_CORPUS_DETACH(t); } // reject: detach inside a macro
void d10() { std::thread{work}.detach(); } // reject: detach of a temporary

} // namespace corpus
