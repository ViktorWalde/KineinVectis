//! O INTEGRADOR: transforma a formula da derivada numa trajetoria.
//!
//! Tres metodos, e a escolha e' do usuario — nao ha' padrao (`arquitetura/34`
//! §2.1). O motivo nao e' rigor: **o metodo muda o numero**. Medido no
//! oscilador amortecido com `dt=0,1` (`roadmaps/31` §11.1):
//!
//! ```text
//! Euler explicito    erro 3,11      <- onze vezes o valor da resposta (-0,276)
//! Euler simpletico   erro 3,96e-2
//! RK4                erro 4,91e-5
//! ```
//!
//! ## A trilha e' AMOSTRADA, e a amostragem aparece
//!
//! Guardar todo passo e' inviavel por ordem de grandeza: um milhao de passos
//! sao 32 MB, quarenta milhoes sao 1,28 GB (`roadmaps/31` §14). A trilha guarda
//! `samples` pontos, e o `sample_every` que sai daqui vai a tela — uma tabela
//! amostrada PARECE completa, e a IDE diz de quanto em quanto ela olhou.
//!
//! ## O que este arquivo NAO faz
//!
//! Nao escolhe metodo, nao escolhe passo, nao corrige passo ruim e nao decide
//! quantas amostras guardar. Ele recebe as quatro coisas e integra.

// RELAXAMENTO REGISTRADO, como a §8 do ARCHITECTURE exige (relaxar strict mode
// SEM registrar motivo e' que e' anti-padrao).
//
// O `clippy::suboptimal_flops` pede `mul_add` em cada estagio do RK4, trocando
// `y + dt / 2.0 * k1` por `(dt / 2.0).mul_add(k1, y)`. Ele tem razao sobre o
// arredondamento — o FMA arredonda uma vez em vez de duas.
//
// Aqui o custo e' maior que o ganho, e por um motivo que vale para ESTE arquivo
// e nao para os outros: **as formulas abaixo sao a transcricao de um metodo de
// livro, e a verificacao delas e' humana**. O gate que prova o integrador e' o
// estudo de ordem de convergencia (`tests/sim_integrador.rs`), e ele so' vale se
// alguem conseguir ler os quatro estagios do RK4 e reconhece-los. Reescritos em
// `mul_add` aninhado, eles deixam de ser reconheciveis.
//
// E a medicao mostra que o ganho nao muda nada onde importa: os testes medem
// ordem 3,85 e 3,99 para o RK4, contra 4 teorico — o erro de TRUNCAMENTO domina
// na faixa util, nao o de arredondamento (`roadmaps/31` §11.3). Onde o
// arredondamento manda, no oraculo do `catalogo.rs`, o `mul_add` ESTA usado.
#![allow(clippy::suboptimal_flops)]

use kinein_protocol::{SimInitial, SimMethod, SimSample};

/// Quanto uma corrida pode custar antes de a IDE recusar a fazer.
///
/// Nao e' teto de gosto: e' o ponto em que a trilha completa passaria de 3,2 GB
/// se alguem guardasse tudo, e onde o interpretador leva minutos. Recusar com o
/// numero na tela e' melhor que travar a IDE — e quem decide reduzir o passo ou
/// a duracao e' o usuario.
pub const TETO_DE_PASSOS: u64 = 100_000_000;

/// O que impede uma corrida de acontecer, ou de terminar.
#[derive(Debug, Clone, PartialEq)]
pub enum ErroDeCorrida {
    /// Passo zero, negativo ou nao-finito.
    PassoInvalido,
    /// Duracao zero, negativa ou nao-finita.
    DuracaoInvalida,
    /// O numero de passos passa do teto.
    CorridaLongaDemais {
        /// Quantos passos a corrida pediria.
        passos: u64,
        /// O teto.
        teto: u64,
    },
    /// A segunda ordem exige `y'(0)`, e ele nao veio.
    FaltaDerivadaInicial,
    /// A conta saiu do dominio dos numeros no meio da integracao.
    ///
    /// Acontece de verdade: passo grande demais num sistema rigido diverge, e o
    /// `f64` chega a `inf` em poucos milhares de passos. Entregar a trilha ate'
    /// ali com cara de resultado seria pior que parar.
    Divergiu {
        /// Em que passo o estado deixou de ser finito.
        passo: u64,
        /// Em que instante.
        t: f64,
    },
}

/// A funcao que o integrador chama a cada avaliacao.
///
/// Recebe `(t, y, dy)` e devolve a derivada pedida pela forma. E' um `FnMut`
/// porque quem a implementa carrega o vetor de avaliacao do `exmex` e o
/// reaproveita entre passos — alocar um `Vec` por passo dominaria o custo.
pub trait Derivada {
    /// Avalia. `None` quando a expressao nao pode ser avaliada.
    fn avaliar(&mut self, t: f64, y: f64, dy: f64) -> Option<f64>;
}

impl<F: FnMut(f64, f64, f64) -> Option<f64>> Derivada for F {
    fn avaliar(&mut self, t: f64, y: f64, dy: f64) -> Option<f64> {
        self(t, y, dy)
    }
}

/// Uma corrida pronta para integrar.
#[derive(Debug, Clone)]
pub struct Corrida {
    /// Passo de tempo.
    pub passo: f64,
    /// Ate' quando.
    pub duracao: f64,
    /// Metodo escolhido pelo usuario.
    pub metodo: SimMethod,
    /// Quantos pontos guardar.
    pub amostras: usize,
    /// Estado inicial.
    pub inicial: SimInitial,
    /// Se a forma e' de segunda ordem (`y'' = f`) ou de primeira (`y' = f`).
    pub segunda_ordem: bool,
}

/// O que saiu da integracao.
#[derive(Debug, Clone)]
pub struct Saida {
    /// Passos dados.
    pub passos: u64,
    /// De quantos em quantos a trilha guardou.
    pub a_cada: u64,
    /// A trilha amostrada.
    pub trilha: Vec<SimSample>,
    /// O estado final, para comparar com a solucao fechada.
    pub final_y: f64,
    /// O instante que a integracao ALCANCOU: `passos * dt`.
    ///
    /// Existe desde 2026-09-06, e ele conserta um defeito que a forma escalar
    /// carregava sem aparecer: o oraculo era perguntado pela `duracao` PEDIDA,
    /// e a corrida para em `passos * dt`. Com `duracao = 10` e `dt = 0,1` os
    /// dois coincidem e nada se via; com uma duracao que nao e' multiplo do
    /// passo, o "erro" mostrado passa a incluir uma diferenca de TEMPO que nao
    /// e' erro de integracao nenhum. Achado na forma vetorial, onde o periodo
    /// da orbita e' irracional e a diferenca fez um RK4 medir ordem 0,81.
    pub t_final: f64,
    /// O maior `|y|` que a corrida atingiu.
    ///
    /// Existe porque um teste de 2026-09-05 achou o buraco: com `dt=5` no
    /// oscilador amortecido a trilha chega a **3,5e209** sem nunca virar `inf`,
    /// e a checagem de `is_finite` deixa passar. Um valor desses num sistema
    /// que deveria decair a zero e' lixo com cara de resultado — a mesma forma
    /// do defeito de estabilidade da EDP (`roadmaps/31` §16.1).
    ///
    /// **A IDE nao ADIVINHA se o crescimento e' fisico**: uma EDO de
    /// crescimento exponencial cresce mesmo, e chutar um teto seria a IDE
    /// decidindo pelo usuario. O que ela faz e' MOSTRAR a magnitude, e — quando
    /// ha' solucao fechada — mostrar o erro, que e' o detector de verdade.
    pub maior_abs: f64,
}

impl Corrida {
    /// Quantos passos esta corrida daria. Publico porque a IDE mostra o numero
    /// ANTES de rodar.
    #[must_use]
    pub fn passos(&self) -> u64 {
        let bruto = self.duracao / self.passo;
        if !bruto.is_finite() || bruto <= 0.0 {
            return 0;
        }
        let arredondado = bruto.round().max(1.0);
        // Acima de 2^53 um `f64` nao representa inteiros consecutivos, entao a
        // conversao deixaria de ser exata. Saturar aqui e' honesto: qualquer
        // valor desses ja' passou do teto de passos por muitas ordens de
        // grandeza, e o `validar` recusa a corrida logo em seguida.
        if arredondado >= 9_007_199_254_740_992.0 {
            return u64::MAX;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            arredondado as u64
        }
    }

    fn validar(&self) -> Result<u64, ErroDeCorrida> {
        if !self.passo.is_finite() || self.passo <= 0.0 {
            return Err(ErroDeCorrida::PassoInvalido);
        }
        if !self.duracao.is_finite() || self.duracao <= 0.0 {
            return Err(ErroDeCorrida::DuracaoInvalida);
        }
        if self.segunda_ordem && self.inicial.dy.is_none() {
            return Err(ErroDeCorrida::FaltaDerivadaInicial);
        }
        let passos = self.passos();
        if passos > TETO_DE_PASSOS {
            return Err(ErroDeCorrida::CorridaLongaDemais {
                passos,
                teto: TETO_DE_PASSOS,
            });
        }
        Ok(passos)
    }

    /// Integra, guardando `amostras` pontos ao longo do caminho.
    pub fn integrar<D: Derivada>(&self, mut f: D) -> Result<Saida, ErroDeCorrida> {
        let passos = self.validar()?;
        let a_cada = if self.amostras == 0 {
            passos.max(1)
        } else {
            (passos / u64::try_from(self.amostras).unwrap_or(u64::MAX)).max(1)
        };

        let dt = self.passo;
        let mut y = self.inicial.y;
        let mut dy = self.inicial.dy.unwrap_or(0.0);
        // `usize::try_from` em vez de `as`: numa maquina de 32 bits o `as`
        // truncaria e a reserva sairia menor que o necessario. O fallback e' o
        // proprio numero de amostras, que ja' e' `usize`.
        let cabem = usize::try_from(passos).map_or(self.amostras, |n| n + 1);
        let mut trilha = Vec::with_capacity(self.amostras.min(cabem) + 1);
        trilha.push(self.amostra(0.0, y, dy));
        let mut maior_abs = y.abs();

        for passo in 0..passos {
            // O `as f64` perde exatidao acima de 2^53 passos — e' um limite
            // teorico bem acima do teto de 1e8 que o `validar` ja' impoe.
            #[allow(clippy::cast_precision_loss)]
            let t = passo as f64 * dt;
            let (posicao, velocidade) = if self.segunda_ordem {
                self.passo_segunda_ordem(&mut f, t, y, dy)?
            } else {
                (self.passo_primeira_ordem(&mut f, t, y)?, 0.0)
            };
            y = posicao;
            dy = velocidade;
            if !y.is_finite() || !dy.is_finite() {
                return Err(ErroDeCorrida::Divergiu {
                    passo: passo + 1,
                    t: t + dt,
                });
            }
            maior_abs = maior_abs.max(y.abs());
            if (passo + 1) % a_cada == 0 {
                trilha.push(self.amostra(t + dt, y, dy));
            }
        }
        // O ultimo instante entra sempre: o usuario pediu `duracao`, e nao
        // "o multiplo de `a_cada` mais proximo dela".
        #[allow(clippy::cast_precision_loss)]
        let t_final = passos as f64 * dt;
        if trilha.last().is_none_or(|u| u.t < t_final) {
            trilha.push(self.amostra(t_final, y, dy));
        }

        Ok(Saida {
            passos,
            a_cada,
            trilha,
            final_y: y,
            t_final,
            maior_abs,
        })
    }

    fn amostra(&self, t: f64, y: f64, dy: f64) -> SimSample {
        SimSample {
            t,
            y,
            dy: self.segunda_ordem.then_some(dy),
        }
    }

    fn passo_primeira_ordem<D: Derivada>(
        &self,
        f: &mut D,
        t: f64,
        y: f64,
    ) -> Result<f64, ErroDeCorrida> {
        let dt = self.passo;
        let erro = || ErroDeCorrida::Divergiu { passo: 0, t };
        Ok(match self.metodo {
            // Os dois Euler coincidem em primeira ordem: nao ha' velocidade
            // separada para atualizar antes da posicao.
            SimMethod::Euler | SimMethod::EulerSymplectic => {
                y + dt * f.avaliar(t, y, 0.0).ok_or_else(erro)?
            }
            SimMethod::Rk4 => {
                let k1 = f.avaliar(t, y, 0.0).ok_or_else(erro)?;
                let k2 = f
                    .avaliar(t + dt / 2.0, y + dt / 2.0 * k1, 0.0)
                    .ok_or_else(erro)?;
                let k3 = f
                    .avaliar(t + dt / 2.0, y + dt / 2.0 * k2, 0.0)
                    .ok_or_else(erro)?;
                let k4 = f.avaliar(t + dt, y + dt * k3, 0.0).ok_or_else(erro)?;
                y + dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4)
            }
        })
    }

    fn passo_segunda_ordem<D: Derivada>(
        &self,
        f: &mut D,
        t: f64,
        y: f64,
        dy: f64,
    ) -> Result<(f64, f64), ErroDeCorrida> {
        let dt = self.passo;
        let erro = || ErroDeCorrida::Divergiu { passo: 0, t };
        Ok(match self.metodo {
            // EXPLICITO: usa a posicao velha para achar a velocidade nova E a
            // posicao nova. E' o que injeta energia num oscilador.
            SimMethod::Euler => {
                let a = f.avaliar(t, y, dy).ok_or_else(erro)?;
                (y + dt * dy, dy + dt * a)
            }
            // SIMPLETICO: atualiza a velocidade primeiro e usa a NOVA para
            // andar com a posicao. Uma linha de diferenca, duas ordens de
            // grandeza de erro a menos no oscilador (roadmaps/31 §11.1).
            SimMethod::EulerSymplectic => {
                let a = f.avaliar(t, y, dy).ok_or_else(erro)?;
                let dy_novo = dy + dt * a;
                (y + dt * dy_novo, dy_novo)
            }
            SimMethod::Rk4 => {
                let (k1y, k1v) = (dy, f.avaliar(t, y, dy).ok_or_else(erro)?);
                let (k2y, k2v) = (
                    dy + dt / 2.0 * k1v,
                    f.avaliar(t + dt / 2.0, y + dt / 2.0 * k1y, dy + dt / 2.0 * k1v)
                        .ok_or_else(erro)?,
                );
                let (k3y, k3v) = (
                    dy + dt / 2.0 * k2v,
                    f.avaliar(t + dt / 2.0, y + dt / 2.0 * k2y, dy + dt / 2.0 * k2v)
                        .ok_or_else(erro)?,
                );
                let (k4y, k4v) = (
                    dy + dt * k3v,
                    f.avaliar(t + dt, y + dt * k3y, dy + dt * k3v)
                        .ok_or_else(erro)?,
                );
                (
                    y + dt / 6.0 * (k1y + 2.0 * k2y + 2.0 * k3y + k4y),
                    dy + dt / 6.0 * (k1v + 2.0 * k2v + 2.0 * k3v + k4v),
                )
            }
        })
    }
}

/// O rotulo que vai a tela junto do resultado.
///
/// Numero sem procedencia mente (`docs/README.md`): a tela nunca mostra o
/// resultado sem dizer com que metodo e com que passo ele foi obtido.
#[must_use]
pub fn rotulo(metodo: SimMethod, passo: f64) -> String {
    let nome = match metodo {
        SimMethod::Euler => "Euler explicito",
        SimMethod::EulerSymplectic => "Euler simpletico",
        SimMethod::Rk4 => "Runge-Kutta 4",
    };
    format!("{nome}, dt = {passo:e}")
}

/// Quantos bytes uma amostra ocupa: `t`, `y` e `dy` em `f64`.
const BYTES_POR_AMOSTRA: u64 = 24;

/// O ponto de virada entre interpretar e compilar, medido nesta maquina em
/// 2026-09-05: `490 ms / 22,75 ns` (`roadmaps/31` §9.3).
pub const VIRADA_COMPILAR: u64 = 21_500_000;

/// A estimativa que a IDE mostra ANTES de rodar.
///
/// Ela nao decide nada — nem o motor, nem o passo, nem a amostragem. Ela
/// informa, e o usuario escolhe (`arquitetura/34` §2.1). Informar nao e decidir.
#[must_use]
pub fn estimar(duracao: f64, passo: f64, amostras: usize) -> kinein_protocol::SimEstimateResult {
    let corrida = Corrida {
        passo,
        duracao,
        metodo: SimMethod::Rk4,
        amostras,
        inicial: SimInitial {
            y: 0.0,
            dy: Some(0.0),
        },
        segunda_ordem: false,
    };
    let passos = corrida.passos();
    let a_cada = if amostras == 0 {
        passos.max(1)
    } else {
        (passos / u64::try_from(amostras).unwrap_or(u64::MAX)).max(1)
    };
    let guardadas = passos.checked_div(a_cada).unwrap_or(0);
    kinein_protocol::SimEstimateResult {
        steps: passos,
        too_many: passos > TETO_DE_PASSOS,
        limit: TETO_DE_PASSOS,
        sample_every: a_cada,
        full_trail_bytes: passos.saturating_mul(BYTES_POR_AMOSTRA),
        sampled_trail_bytes: guardadas.saturating_mul(BYTES_POR_AMOSTRA),
        compiling_would_pay: passos > VIRADA_COMPILAR,
    }
}
