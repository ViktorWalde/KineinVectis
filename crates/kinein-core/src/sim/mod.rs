//! O dominio `sim`: simulacao por conceito.
//!
//! Desenho completo em `docs/arquitetura/34-simulacao-por-conceito.md`, medicao
//! que o sustenta em `docs/roadmaps/31-simulacao-fisica-matematica.md` §8 a §19.
//!
//! ## O corte
//!
//! ```text
//! catalogo.rs        a TABELA de conceitos — a camada de CIMA, que cresce por
//!                    ENTRADA. Nao guarda formula: a formula e' do usuario
//! formula.rs         inspecionar, checar contra o conceito e avaliar. Onde
//!                    mora a ligacao explicita, que impede a IDE de adivinhar
//! integrador.rs      o passo no tempo das formas ESCALARES (EDO_1 e EDO_2)
//! sistema.rs         o passo no tempo da forma VETORIAL (SISTEMA_EDO)
//! corrida.rs         a cola escalar: formula + integrador + oraculo
//! corrida_sistema.rs a cola vetorial: n formulas + sistema + oraculo +
//!                    invariantes
//! exata.rs           as SOLUCOES FECHADAS. Saiu do catalogo em 2026-09-06 por
//!                    responsabilidade: tabela e matematica mudam por motivos
//!                    diferentes
//! invariante.rs      as grandezas que a fisica CONSERVA, e a deriva delas — o
//!                    sinal de exatidao que existe quando nao ha' forma fechada
//! oraculo.rs         resolve a equacao que o usuario DIGITOU, num processo
//!                    externo. E' quem faz a coluna `exato` parar de responder
//!                    por outra pergunta (2026-09-10)
//! persistencia.rs    o que o autor montou, em .kinein/simulacoes/
//! ```
//!
//! ## Onde as cinco formas estao, em 2026-09-06
//!
//! ```text
//! ALGEBRICA     COM MOTOR   avaliacao direta (2026-09-05)
//! EDO_1         COM MOTOR   integrador escalar (2026-09-05)
//! EDO_2         COM MOTOR   integrador escalar (2026-09-05)
//! SISTEMA_EDO   COM MOTOR   integrador vetorial (2026-09-06)
//! EDP           SEM MOTOR   e' a proxima decidida pelo autor, DEPOIS do
//!                           oraculo. Ela exige o motor COMPILADO, que nao
//!                           existe: medido em 2026-09-06, o interpretado e'
//!                           90,3x mais lento por celula, e 201x201 por um
//!                           segundo de fisica leva SEIS MINUTOS
//! ```
//!
//! **Conceito de forma sem motor nao entra no catalogo**: ele apareceria como
//! oferta que a IDE nao cumpre.

pub mod catalogo;
pub mod corrida;
pub mod corrida_sistema;
pub mod entradas;
pub mod entradas_sistema;
pub mod exata;
pub mod formula;
pub mod integrador;
pub mod invariante;
pub mod oraculo;
pub mod persistencia;
pub mod sistema;
