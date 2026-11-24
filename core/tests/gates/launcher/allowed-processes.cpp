// Corpus of gate.launcher: the correct forms of "processes only through the launcher".
//
// Documented in: docs/engineering/gates-and-effectiveness.md
//
// No line of this file may receive the diagnostic. The lines marked with "accept:" are the forms
// the gate must accept, including the hard ones: the same name in another namespace, text,
// comment, macro and a similar prefix.
#include <chrono>
#include <csignal>
#include <cstdlib>
#include <filesystem>
#include <spawn.h>
#include <sys/wait.h>
#include <unistd.h>

namespace kinein::sys {
int fork();
int system(const char* command);
} // namespace kinein::sys

class Process
{
public:
    void start();
    static int execute(const char* program);
};

struct Popen
{
    int operator()() const { return 0; }
};

namespace corpus {

int a01() { return kinein::sys::fork(); } // accept: own function named fork, in another namespace
int a02() { return kinein::sys::system("x"); } // accept: own function named system
void a03(Process& process) { process.start(); } // accept: start of an own class
int a04() { return Process::execute("x"); } // accept: static execute of an own class
const char* a05() { return "fork() system() popen() execvp()"; } // accept: string literal
int a06() { return 0; /* fork(); system("x"); */ } // accept: comment
int a07() { posix_spawn_file_actions_t actions{}; return posix_spawn_file_actions_init(&actions); } // accept: posix_spawn prefix
auto a08() { return std::chrono::system_clock::now(); } // accept: system_clock
bool a09() { return std::filesystem::exists("/bin/true"); } // accept: filesystem
int a10() { int status{}; return waitpid(-1, &status, WNOHANG); } // accept: waiting creates no process
int a11() { return kill(0, 0); } // accept: a signal creates no process
pid_t a12() { return getpid(); } // accept: getpid
int a13() { int system = 1; return system; } // accept: variable named system
#define KINEIN_CORPUS_SYSTEM_NAME "linux"
const char* a14() { return KINEIN_CORPUS_SYSTEM_NAME; } // accept: macro with system in its name
int a15() { return std::atoi("1"); } // accept: another <cstdlib> function
int a16() { return Popen{}(); } // accept: type named Popen

} // namespace corpus
