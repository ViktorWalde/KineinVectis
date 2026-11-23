// Corpus of gate.launcher: the planted defects of "processes only through the launcher".
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
//
// Each line marked with "reject:" is a defect. clang-tidy, with the configuration of
// core/.clang-tidy, must flag the call exactly on that line and on no other line. The file is
// only analyzed: it belongs to no target, and nothing runs.
#include <cstdio>
#include <cstdlib>
#include <pty.h>
#include <sched.h>
#include <spawn.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <wordexp.h>

// Doubles with the qualified names of the Qt and Windows APIs: the check matches
// the name, and the corpus does not depend on Qt or Windows being installed.
class QProcess
{
public:
    void start();
    void startCommand(const char* command);
    bool open();
    static bool startDetached(const char* program);
    static int execute(const char* program);
};
extern "C" int CreateProcessA(void* info);
extern "C" int CreateProcessW(void* info);
extern "C" int ShellExecuteW(void* info);
extern "C" int WinExec(const char* line);
extern "C" int _wsystem(const wchar_t* line);
extern "C" int _spawnvp(int mode);

namespace corpus {

char program[] = "true";
char* argv[] = {program, nullptr};
char* envp[] = {nullptr};

template <typename Function>
int call(Function function)
{
    return function();
}

int f01() { return fork(); } // reject: direct fork
int f02() { return ::vfork(); } // reject: qualified vfork
int f03() { return _Fork(); } // reject: _Fork, glibc 2.34
int f04() { return execl("/bin/true", "true", nullptr); } // reject: execl
int f05() { return execle("/bin/true", "true", nullptr, envp); } // reject: execle
int f06() { return execlp("true", "true", nullptr); } // reject: execlp
int f07() { return execv("/bin/true", argv); } // reject: execv
int f08() { return execve("/bin/true", argv, envp); } // reject: execve
int f09() { return execvp("true", argv); } // reject: execvp
int f10() { return execvpe("true", argv, envp); } // reject: execvpe
int f11() { return execveat(0, "true", argv, envp, 0); } // reject: execveat
int f12() { return fexecve(0, argv, envp); } // reject: fexecve
int f13() { pid_t pid{}; return posix_spawn(&pid, "/bin/true", nullptr, nullptr, argv, envp); } // reject: posix_spawn
int f14() { pid_t pid{}; return posix_spawnp(&pid, "true", nullptr, nullptr, argv, envp); } // reject: posix_spawnp
int f15() { int fd{}; return pidfd_spawn(&fd, "/bin/true", nullptr, nullptr, argv, envp); } // reject: pidfd_spawn, glibc 2.39
int f16() { int fd{}; return pidfd_spawnp(&fd, "true", nullptr, nullptr, argv, envp); } // reject: pidfd_spawnp
int f17() { return std::system("true"); } // reject: std::system
int f18() { return system("true"); } // reject: global system
FILE* f19() { return popen("true", "r"); } // reject: popen
long f20() { return syscall(SYS_clone, 0); } // reject: raw syscall
int f21() { return clone(nullptr, nullptr, 0, nullptr); } // reject: clone
int f22() { int master{}; return forkpty(&master, nullptr, nullptr, nullptr); } // reject: forkpty
int f23() { return daemon(0, 0); } // reject: daemon, which forks
int f24() { wordexp_t words{}; return wordexp("$(true)", &words, 0); } // reject: wordexp runs command substitution
int f25() { auto* spawn = &fork; return spawn(); } // reject: pointer to the function
#define KINEIN_CORPUS_SPAWN() fork()
int f26() { return KINEIN_CORPUS_SPAWN(); } // reject: hidden in a macro
namespace by_using { using ::popen; FILE* f27() { return popen("true", "r"); } } // reject: via using
int f28() { return call(fork); } // reject: passed as an argument
int f29() { return [] { return vfork(); }(); } // reject: inside a lambda
void f30(QProcess& process) { process.start(); } // reject: QProcess::start
void f31(QProcess& process) { process.startCommand("true"); } // reject: QProcess::startCommand
bool f32(QProcess& process) { return process.open(); } // reject: QProcess::open starts the program
bool f33() { return QProcess::startDetached("true"); } // reject: QProcess::startDetached
int f34() { return QProcess::execute("true"); } // reject: QProcess::execute
int f35() { return CreateProcessW(nullptr); } // reject: CreateProcessW
int f36() { return CreateProcessA(nullptr); } // reject: CreateProcessA
int f37() { return ShellExecuteW(nullptr); } // reject: ShellExecuteW
int f38() { return WinExec("true"); } // reject: WinExec
int f39() { return _wsystem(L"true"); } // reject: _wsystem
int f40() { return _spawnvp(0); } // reject: _spawnvp

} // namespace corpus
