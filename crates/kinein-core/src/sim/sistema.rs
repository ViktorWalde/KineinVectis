//! O INTEGRADOR VETORIAL: `dY/dt = F(t, Y)`, com `Y` de `n` componentes.
//!
//! Esta e' a quinta forma do motor a ganhar codigo, e a decisao que a governa
//! esta' na `arquitetura/34` §13: **ela e' de PRIMEIRA ORDEM**. Um sistema de
//! segunda ordem entra escrevendo as velocidades como componentes, que e' a
//! reducao padrao — e o motivo e' um mecanismo, nao dois.
//!
//! ## O simpletico exige um pareamento, e ele nao e' deduzido
//!
//! O metodo simpletico e' *atualize a velocidade, depois ande com a posicao
//! usando a velocidade NOVA*. Isso exige saber qual componente e' posicao de
//! qual velocidade — e num `dY/dt = F(t,Y)` generico esse par **nao existe**.
//!
//! Deduzir o par (por nome, por ordem) e' a deducao que erra calada, e o
//! principio da §2.1 a proibe. Entao **o conceito declara** o pareamento, e um
//! conceito que nao declara simplesmente **nao oferece o metodo** — com o
//! motivo dito, em vez de integrar outra coisa em silencio.
//!
//! O que a medicao paga por essa complicacao (`../roadmaps/31` §19.1.1), numa
//! orbita circular de raio verdadeiro 1, `dt=0,01`, dez voltas:
//!
//! ```text
//! Euler explicito    raio 1,647957    deriva de energia 2,032e-01
//! Euler simpletico   raio 1,000024    deriva de energia 2,800e-10
//! ```
//!
//! ## O que este arquivo NAO faz
//!
//! Nao escolhe metodo, passo, amostragem nem estado inicial. Recebe os quatro e
//! integra.

// RELAXAMENTO REGISTRADO, pelo mesmo motivo do `integrador.rs` e com a mesma
// forca: as expressoes abaixo sao a transcricao vetorial de um metodo de livro,
// e o gate que as prova e' um estudo de ordem de convergencia que so' vale se
// alguem conseguir ler os quatro estagios e reconhece-los. Em `mul_add`
// aninhado eles deixam de ser reconheciveis. Onde o arredondamento manda — no
// oraculo do `exata.rs` e nos invariantes — o `mul_add` ESTA usado.
#![allow(clippy::suboptimal_flops)]

use kinein_protocol::{SimMethod, SimPair, SimSystemSample};

use super::integrador::{ErroDeCorrida, TETO_DE_PASSOS};

/// A funcao que o integrador chama a cada avaliacao de UM componente.
///
/// Recebe `(t, Y)` e escreve `dY/dt` em `saida`. E' `FnMut` porque quem a
/// implementa carrega os vetores de avaliacao do avaliador e os reaproveita
/// entre passos: alocar por passo dominaria o custo numa corrida de milhoes.
pub trait Campo {
    /// Avalia as `n` derivadas. `false` quando alguma nao pode ser avaliada.
    fn avaliar(&mut self, t: f64, estado: &[f64], saida: &mut [f64]) -> bool;
}

impl<F: FnMut(f64, &[f64], &mut [f64]) -> bool> Campo for F {
    fn avaliar(&mut self, t: f64, estado: &[f64], saida: &mut [f64]) -> bool {
        self(t, estado, saida)
    }
}

/// O que impede uma corrida de sistema de acontecer.
#[derive(Debug, Clone, PartialEq)]
pub enum ErroDeSistema {
    /// Um problema que a forma escalar tambem tem.
    Corrida(ErroDeCorrida),
    /// O metodo simpletico foi pedido num conceito sem pareamento declarado.
    ///
    /// Nao e' limitacao a contornar: sem o par, o metodo **nao esta' definido**.
    SemPareamento,
    /// O estado inicial nao tem um valor por componente.
    EstadoIncompleto {
        /// Quantos componentes o conceito declara.
        esperado: usize,
        /// Quantos vieram.
        recebido: usize,
    },
}

/// Uma corrida de sistema, pronta para integrar.
#[derive(Debug, Clone)]
pub struct CorridaSistema {
    /// Passo de tempo.
    pub passo: f64,
    /// Ate' quando.
    pub duracao: f64,
    /// Metodo escolhido pelo usuario.
    pub metodo: SimMethod,
    /// Quantos pontos guardar.
    pub amostras: usize,
    /// Estado em t = 0, um valor por componente.
    pub inicial: Vec<f64>,
    /// Os pares posicao/velocidade que o CONCEITO declara. Vazio quando ele
    /// nao declara nenhum — e ai o simpletico e' recusado.
    pub pareamento: Vec<SimPair>,
}

/// O que saiu da integracao.
#[derive(Debug, Clone)]
pub struct SaidaSistema {
    /// Passos dados.
    pub passos: u64,
    /// De quantos em quantos a trilha guardou.
    pub a_cada: u64,
    /// A trilha amostrada.
    pub trilha: Vec<SimSystemSample>,
    /// O estado final, para comparar com a solucao fechada.
    pub estado_final: Vec<f64>,
    /// O instante que a integracao ALCANCOU: `passos * dt`.
    ///
    /// Nao e' a `duracao` pedida, e a diferenca importa. Com `duracao = 2*pi` e
    /// `dt = 0,01` a corrida da' 628 passos e para em 6,28 — a 3,2e-3 do
    /// `2*pi = 6,28318...`. Perguntar ao oraculo pelo instante PEDIDO em vez do
    /// alcancado mistura erro de integracao com uma diferenca de tempo que nao
    /// e' erro nenhum, e foi o que fez o estudo de ordem medir 0,81 num RK4.
    pub t_final: f64,
}

impl CorridaSistema {
    /// Quantos passos esta corrida daria. Publico porque a IDE mostra o numero
    /// ANTES de rodar.
    #[must_use]
    pub fn passos(&self) -> u64 {
        super::integrador::Corrida {
            passo: self.passo,
            duracao: self.duracao,
            metodo: self.metodo,
            amostras: self.amostras,
            inicial: kinein_protocol::SimInitial { y: 0.0, dy: None },
            segunda_ordem: false,
        }
        .passos()
    }

    fn validar(&self, componentes: usize) -> Result<u64, ErroDeSistema> {
        if self.inicial.len() != componentes {
            return Err(ErroDeSistema::EstadoIncompleto {
                esperado: componentes,
                recebido: self.inicial.len(),
            });
        }
        if self.metodo == SimMethod::EulerSymplectic && self.pareamento.is_empty() {
            return Err(ErroDeSistema::SemPareamento);
        }
        if !self.passo.is_finite() || self.passo <= 0.0 {
            return Err(ErroDeSistema::Corrida(ErroDeCorrida::PassoInvalido));
        }
        if !self.duracao.is_finite() || self.duracao <= 0.0 {
            return Err(ErroDeSistema::Corrida(ErroDeCorrida::DuracaoInvalida));
        }
        let passos = self.passos();
        if passos > TETO_DE_PASSOS {
            return Err(ErroDeSistema::Corrida(ErroDeCorrida::CorridaLongaDemais {
                passos,
                teto: TETO_DE_PASSOS,
            }));
        }
        Ok(passos)
    }

    /// Integra, guardando `amostras` pontos ao longo do caminho.
    pub fn integrar<C: Campo>(
        &self,
        componentes: usize,
        mut f: C,
    ) -> Result<SaidaSistema, ErroDeSistema> {
        let passos = self.validar(componentes)?;
        let a_cada = if self.amostras == 0 {
            passos.max(1)
        } else {
            (passos / u64::try_from(self.amostras).unwrap_or(u64::MAX)).max(1)
        };

        let dt = self.passo;
        let mut y = self.inicial.clone();
        let mut k1 = vec![0.0; componentes];
        let mut k2 = vec![0.0; componentes];
        let mut k3 = vec![0.0; componentes];
        let mut k4 = vec![0.0; componentes];
        let mut tmp = vec![0.0; componentes];

        let cabem = usize::try_from(passos).map_or(self.amostras, |n| n + 1);
        let mut trilha = Vec::with_capacity(self.amostras.min(cabem) + 1);
        trilha.push(SimSystemSample {
            t: 0.0,
            values: y.clone(),
        });

        for passo in 0..passos {
            #[allow(clippy::cast_precision_loss)]
            let t = passo as f64 * dt;
            let avaliou = match self.metodo {
                SimMethod::Euler => self.passo_euler(&mut f, t, &mut y, &mut k1),
                SimMethod::EulerSymplectic => self.passo_simpletico(&mut f, t, &mut y, &mut k1),
                SimMethod::Rk4 => self.passo_rk4(
                    &mut f,
                    t,
                    &mut y,
                    (&mut k1, &mut k2, &mut k3, &mut k4),
                    &mut tmp,
                ),
            };
            if !avaliou || y.iter().any(|v| !v.is_finite()) {
                return Err(ErroDeSistema::Corrida(ErroDeCorrida::Divergiu {
                    passo: passo + 1,
                    t: t + dt,
                }));
            }
            if (passo + 1) % a_cada == 0 {
                trilha.push(SimSystemSample {
                    t: t + dt,
                    values: y.clone(),
                });
            }
        }

        // O ultimo instante entra sempre: o usuario pediu `duracao`, e nao "o
        // multiplo de `a_cada` mais proximo dela".
        #[allow(clippy::cast_precision_loss)]
        let t_final = passos as f64 * dt;
        if trilha.last().is_none_or(|u| u.t < t_final) {
            trilha.push(SimSystemSample {
                t: t_final,
                values: y.clone(),
            });
        }

        Ok(SaidaSistema {
            passos,
            a_cada,
            trilha,
            estado_final: y,
            t_final,
        })
    }

    /// EXPLICITO: todas as componentes andam com a derivada avaliada no estado
    /// VELHO. E' o que injeta energia num sistema orbitando.
    fn passo_euler<C: Campo>(&self, f: &mut C, t: f64, y: &mut [f64], k: &mut [f64]) -> bool {
        if !f.avaliar(t, y, k) {
            return false;
        }
        for (valor, derivada) in y.iter_mut().zip(k.iter()) {
            *valor += self.passo * derivada;
        }
        true
    }

    /// SIMPLETICO: atualiza a VELOCIDADE de cada par primeiro, e move a posicao
    /// com a velocidade NOVA. As componentes fora de qualquer par andam como no
    /// explicito — elas nao tem par por definicao.
    fn passo_simpletico<C: Campo>(&self, f: &mut C, t: f64, y: &mut [f64], k: &mut [f64]) -> bool {
        if !f.avaliar(t, y, k) {
            return false;
        }
        let dt = self.passo;
        let mut pareada = vec![false; y.len()];
        for par in &self.pareamento {
            if par.first >= y.len() || par.second >= y.len() {
                continue;
            }
            pareada[par.first] = true;
            pareada[par.second] = true;
            // A velocidade primeiro...
            y[par.second] += dt * k[par.second];
            // ...e a posicao com a velocidade NOVA. Esta e' a unica linha que
            // separa este metodo do explicito, e ela vale oito ordens de
            // grandeza na energia da orbita.
            y[par.first] += dt * y[par.second];
        }
        for (indice, valor) in y.iter_mut().enumerate() {
            if !pareada[indice] {
                *valor += dt * k[indice];
            }
        }
        true
    }

    /// Runge-Kutta classico de quarta ordem, componente a componente.
    fn passo_rk4<C: Campo>(
        &self,
        f: &mut C,
        t: f64,
        y: &mut [f64],
        k: (&mut [f64], &mut [f64], &mut [f64], &mut [f64]),
        tmp: &mut [f64],
    ) -> bool {
        let dt = self.passo;
        let (k1, k2, k3, k4) = k;
        if !f.avaliar(t, y, k1) {
            return false;
        }
        for (destino, (valor, derivada)) in tmp.iter_mut().zip(y.iter().zip(k1.iter())) {
            *destino = valor + dt / 2.0 * derivada;
        }
        if !f.avaliar(t + dt / 2.0, tmp, k2) {
            return false;
        }
        for (destino, (valor, derivada)) in tmp.iter_mut().zip(y.iter().zip(k2.iter())) {
            *destino = valor + dt / 2.0 * derivada;
        }
        if !f.avaliar(t + dt / 2.0, tmp, k3) {
            return false;
        }
        for (destino, (valor, derivada)) in tmp.iter_mut().zip(y.iter().zip(k3.iter())) {
            *destino = valor + dt * derivada;
        }
        if !f.avaliar(t + dt, tmp, k4) {
            return false;
        }
        for (indice, valor) in y.iter_mut().enumerate() {
            *valor += dt / 6.0 * (k1[indice] + 2.0 * k2[indice] + 2.0 * k3[indice] + k4[indice]);
        }
        true
    }
}
