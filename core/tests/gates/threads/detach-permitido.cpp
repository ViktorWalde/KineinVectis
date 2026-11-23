// Corpus do gate "thread nunca solta" (CP.26): as formas corretas.
// (docs/engenharia/gates-e-medida-de-eficacia.md)
//
// Nenhuma linha deste arquivo pode receber o diagnostico. As marcadas com
// "aceita:" sao as formas que o gate tem de aceitar, inclusive as dificeis:
// metodo detach de classe propria, texto, comentario e variavel com o nome.
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

void a01(Session& session) { session.detach(); } // aceita: detach de classe propria
void a02() { kinein::corpus_support::detach(); } // aceita: funcao propria chamada detach
void a03() { std::jthread t(work); } // aceita: jthread junta no destrutor
void a04() { std::thread t(work); t.join(); } // aceita: thread juntada
void a05() { std::jthread t(work_until); t.request_stop(); } // aceita: cancelamento pelo stop_token
const char* a06() { return "t.detach();"; } // aceita: texto literal
void a07() { /* t.detach(); */ } // aceita: comentario
int a08() { int detach = 1; return detach; } // aceita: variavel chamada detach
unsigned a09() { return std::thread::hardware_concurrency(); } // aceita: outro membro de std::thread
std::thread::id a10() { return std::this_thread::get_id(); } // aceita: identificador da thread

} // namespace corpus
