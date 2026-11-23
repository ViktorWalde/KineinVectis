// Corpus do gate "processo so pelo lancador": as formas corretas.
// (docs/engenharia/gates-e-medida-de-eficacia.md)
//
// Nenhuma linha deste arquivo pode receber o diagnostico. As marcadas com
// "aceita:" sao as formas que o gate tem de aceitar, inclusive as dificeis:
// mesmo nome em outro namespace, texto, comentario, macro e prefixo parecido.
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

int a01() { return kinein::sys::fork(); } // aceita: funcao propria chamada fork, em outro namespace
int a02() { return kinein::sys::system("x"); } // aceita: funcao propria chamada system
void a03(Process& process) { process.start(); } // aceita: start de classe propria
int a04() { return Process::execute("x"); } // aceita: execute estatico de classe propria
const char* a05() { return "fork() system() popen() execvp()"; } // aceita: texto literal
int a06() { return 0; /* fork(); system("x"); */ } // aceita: comentario
int a07() { posix_spawn_file_actions_t actions{}; return posix_spawn_file_actions_init(&actions); } // aceita: prefixo posix_spawn
auto a08() { return std::chrono::system_clock::now(); } // aceita: system_clock
bool a09() { return std::filesystem::exists("/bin/true"); } // aceita: filesystem
int a10() { int status{}; return waitpid(-1, &status, WNOHANG); } // aceita: esperar nao cria processo
int a11() { return kill(0, 0); } // aceita: sinal nao cria processo
pid_t a12() { return getpid(); } // aceita: getpid
int a13() { int system = 1; return system; } // aceita: variavel chamada system
#define KINEIN_CORPUS_SYSTEM_NAME "linux"
const char* a14() { return KINEIN_CORPUS_SYSTEM_NAME; } // aceita: macro com system no nome
int a15() { return std::atoi("1"); } // aceita: outra funcao de <cstdlib>
int a16() { return Popen{}(); } // aceita: tipo chamado Popen

} // namespace corpus
