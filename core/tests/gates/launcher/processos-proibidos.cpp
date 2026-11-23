// Corpus do gate "processo so pelo lancador": os defeitos plantados.
// (docs/engenharia/gates-e-medida-de-eficacia.md)
//
// Cada linha marcada com "reprova:" e' um defeito. O clang-tidy, com a
// configuracao do core, tem de acusar a chamada exatamente nela, e em nenhuma
// outra linha. O arquivo so e' analisado: nao entra em alvo nenhum, nada
// executa.
#include <cstdio>
#include <cstdlib>
#include <pty.h>
#include <sched.h>
#include <spawn.h>
#include <sys/syscall.h>
#include <unistd.h>
#include <wordexp.h>

// Dubles com o nome qualificado das APIs do Qt e do Windows: a checagem casa o
// nome, e o corpus nao depende do Qt nem do Windows instalados.
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

int f01() { return fork(); } // reprova: fork direto
int f02() { return ::vfork(); } // reprova: vfork qualificado
int f03() { return _Fork(); } // reprova: _Fork, glibc 2.34
int f04() { return execl("/bin/true", "true", nullptr); } // reprova: execl
int f05() { return execle("/bin/true", "true", nullptr, envp); } // reprova: execle
int f06() { return execlp("true", "true", nullptr); } // reprova: execlp
int f07() { return execv("/bin/true", argv); } // reprova: execv
int f08() { return execve("/bin/true", argv, envp); } // reprova: execve
int f09() { return execvp("true", argv); } // reprova: execvp
int f10() { return execvpe("true", argv, envp); } // reprova: execvpe
int f11() { return execveat(0, "true", argv, envp, 0); } // reprova: execveat
int f12() { return fexecve(0, argv, envp); } // reprova: fexecve
int f13() { pid_t pid{}; return posix_spawn(&pid, "/bin/true", nullptr, nullptr, argv, envp); } // reprova: posix_spawn
int f14() { pid_t pid{}; return posix_spawnp(&pid, "true", nullptr, nullptr, argv, envp); } // reprova: posix_spawnp
int f15() { int fd{}; return pidfd_spawn(&fd, "/bin/true", nullptr, nullptr, argv, envp); } // reprova: pidfd_spawn, glibc 2.39
int f16() { int fd{}; return pidfd_spawnp(&fd, "true", nullptr, nullptr, argv, envp); } // reprova: pidfd_spawnp
int f17() { return std::system("true"); } // reprova: std::system
int f18() { return system("true"); } // reprova: system global
FILE* f19() { return popen("true", "r"); } // reprova: popen
long f20() { return syscall(SYS_clone, 0); } // reprova: syscall cru
int f21() { return clone(nullptr, nullptr, 0, nullptr); } // reprova: clone
int f22() { int master{}; return forkpty(&master, nullptr, nullptr, nullptr); } // reprova: forkpty
int f23() { return daemon(0, 0); } // reprova: daemon, que faz fork
int f24() { wordexp_t words{}; return wordexp("$(true)", &words, 0); } // reprova: wordexp executa substituicao de comando
int f25() { auto* spawn = &fork; return spawn(); } // reprova: ponteiro para a funcao
#define KINEIN_CORPUS_SPAWN() fork()
int f26() { return KINEIN_CORPUS_SPAWN(); } // reprova: escondido numa macro
namespace by_using { using ::popen; FILE* f27() { return popen("true", "r"); } } // reprova: via using
int f28() { return call(fork); } // reprova: passada como argumento
int f29() { return [] { return vfork(); }(); } // reprova: dentro de lambda
void f30(QProcess& process) { process.start(); } // reprova: QProcess::start
void f31(QProcess& process) { process.startCommand("true"); } // reprova: QProcess::startCommand
bool f32(QProcess& process) { return process.open(); } // reprova: QProcess::open inicia o programa
bool f33() { return QProcess::startDetached("true"); } // reprova: QProcess::startDetached
int f34() { return QProcess::execute("true"); } // reprova: QProcess::execute
int f35() { return CreateProcessW(nullptr); } // reprova: CreateProcessW
int f36() { return CreateProcessA(nullptr); } // reprova: CreateProcessA
int f37() { return ShellExecuteW(nullptr); } // reprova: ShellExecuteW
int f38() { return WinExec("true"); } // reprova: WinExec
int f39() { return _wsystem(L"true"); } // reprova: _wsystem
int f40() { return _spawnvp(0); } // reprova: _spawnvp

} // namespace corpus
