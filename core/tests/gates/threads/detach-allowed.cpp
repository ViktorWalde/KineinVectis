// Corpus of gate.detach: the correct forms of "no detached thread".
//
// Documented in: docs/engineering/gates-and-effectiveness.md
//
// The rule is C++ Core Guidelines CP.26. No line of this file may receive the diagnostic. The
// lines marked with "accept:" are the forms the gate must accept, including the hard ones: a
// detach method of an own class, text, comment and a variable with the name.
#include <stop_token>
#include <thread>

namespace kinein::corpus_support {
void detach();
} // namespace kinein::corpus_support

class Session
{
public:
    void detach();
};

namespace corpus {

void work();
void work_until(std::stop_token stop);

void a01(Session& session) { session.detach(); } // accept: detach of an own class
void a02() { kinein::corpus_support::detach(); } // accept: own function named detach
void a03() { std::jthread t(work); } // accept: jthread joins in its destructor
void a04() { std::thread t(work); t.join(); } // accept: joined thread
void a05() { std::jthread t(work_until); t.request_stop(); } // accept: cancellation through the stop_token
const char* a06() { return "t.detach();"; } // accept: string literal
void a07() { /* t.detach(); */ } // accept: comment
int a08() { int detach = 1; return detach; } // accept: variable named detach
unsigned a09() { return std::thread::hardware_concurrency(); } // accept: another member of std::thread
std::thread::id a10() { return std::this_thread::get_id(); } // accept: thread id

} // namespace corpus
