//! O passo a passo OFICIAL de instalacao, por ferramenta e por familia.
//!
//! REGRA DESTE ARQUIVO, e ela nao e' negociavel: **nenhum comando aqui foi
//! escrito por quem programou a IDE.** Cada um foi copiado da documentacao
//! oficial do proprio projeto, e cada entrada carrega a URL e a data em que
//! foi conferida. Se a fonte nao documenta uma familia, a entrada NAO EXISTE
//! para aquela familia — a IDE mostra o link e diz que nao tem passo a passo,
//! em vez de traduzir um comando de outra distro.
//!
//! POR QUE ESSA REGRA. O `tools.rs` registrou em 2026-08 que sugerir instalacao
//! por distro seria "palpite disfarcado de instrucao". A decisao continua
//! valendo contra PALPITE; o que este arquivo faz e' o oposto — citar. A
//! diferenca entre as duas coisas e' a fonte, e por isso a fonte e' campo
//! obrigatorio.
//!
//! A IDE NAO RODA NADA SOZINHA. Os comandos pedem `sudo`, e um instalador
//! silencioso com privilegio de root dentro de um editor de texto e' o tipo de
//! poder que ninguem pediu. O botao "Instalar" ESCREVE os comandos no terminal
//! da propria IDE, onde o autor ve' cada linha e responde o prompt de senha.

/// Um passo do guia: o que ele faz e o comando exato.
#[derive(Debug, Clone, Copy)]
pub(super) struct Step {
    /// O que este passo faz, em uma frase.
    pub(super) explanation: &'static str,
    /// O comando, exatamente como a fonte oficial o escreve.
    pub(super) command: &'static str,
}

/// Guia de instalacao de uma ferramenta numa familia de distro.
#[derive(Debug, Clone, Copy)]
pub(super) struct Guide {
    /// Id da ferramenta (`postgresql`, `timescaledb`, `grafana`, `sqlite`).
    pub(super) tool: &'static str,
    /// Familia a que este guia se aplica (`debian`, `redhat`, ...).
    pub(super) family: &'static str,
    /// Passos, na ordem em que a fonte os apresenta.
    pub(super) steps: &'static [Step],
    /// A pagina oficial de onde estes comandos foram copiados.
    pub(super) source_url: &'static str,
    /// Quando esta entrada foi conferida na fonte (ISO).
    pub(super) checked_at: &'static str,
}

/// Nome e site oficial de cada ferramenta que a IDE sabe instalar.
#[derive(Debug, Clone, Copy)]
pub(super) struct Tool {
    pub(super) id: &'static str,
    pub(super) name: &'static str,
    /// O que ela faz, para quem nunca ouviu falar.
    pub(super) summary: &'static str,
    /// Site oficial — mostrado SEMPRE, inclusive quando nao ha' guia.
    pub(super) website: &'static str,
    /// Binario que prova que ela ja' esta instalada.
    pub(super) probe_binary: &'static str,
}

pub(super) static TOOLS: &[Tool] = &[
    Tool {
        id: "postgresql",
        name: "PostgreSQL",
        summary: "Banco relacional. E' o servidor a que o painel de banco da IDE se conecta.",
        website: "https://www.postgresql.org/download/linux/",
        probe_binary: "psql",
    },
    Tool {
        id: "timescaledb",
        name: "TimescaleDB",
        summary: "Extensao do PostgreSQL para series temporais. Fala o mesmo protocolo, \
                  entao o mesmo perfil de conexao serve.",
        website: "https://www.tigerdata.com/docs/self-hosted/latest/install/installation-linux",
        probe_binary: "timescaledb-tune",
    },
    // Python (bloco B do roadmaps/41, 2026-09-12). As fontes oficiais destas
    // tres sao agnosticas de distro (pipx / uv tool / instalador da Astral): o
    // guia vale para a familia `any`, e a familia especifica ganha quando a
    // fonte a documenta (ruff no Arch e no openSUSE).
    Tool {
        id: "pipx",
        name: "pipx",
        summary: "Instala ferramentas Python isoladas, cada uma no proprio ambiente. E' o \
                  caminho oficial do uv, do ruff e do poetry numa distro que segue a PEP 668.",
        website: "https://pipx.pypa.io/stable/how-to/install-pipx.html",
        probe_binary: "pipx",
    },
    Tool {
        id: "uv",
        name: "uv",
        summary: "Ambientes e pacotes Python, rapido. E' com ele que a IDE cria o .venv do \
                  projeto num clique (`uv venv .venv`).",
        website: "https://docs.astral.sh/uv/getting-started/installation/",
        probe_binary: "uv",
    },
    Tool {
        id: "ruff",
        name: "ruff",
        summary: "Lint e formato de Python num programa so'. Na IDE vira diagnostico, correcao \
                  (Alt+Enter) e formatar.",
        website: "https://docs.astral.sh/ruff/installation/",
        probe_binary: "ruff",
    },
    Tool {
        id: "basedpyright",
        name: "basedpyright",
        summary: "Language server de Python (completar, ir para definicao, tipos, rename). \
                  Pelo PyPI, sem Node; a IDE o sobe com o interpretador do projeto.",
        website: "https://docs.basedpyright.com/latest/installation/command-line-and-language-server/",
        probe_binary: "basedpyright-langserver",
    },
    Tool {
        id: "bear",
        name: "Bear",
        summary: "Gera o compile_commands.json de um Makefile puro interceptando o compilador \
                  (`bear -- make`): e' o que faz o clangd entender um projeto sem CMake. \
                  O build da IDE o usa sozinho quando ele existe.",
        website: "https://github.com/rizsotto/Bear",
        probe_binary: "bear",
    },
    Tool {
        id: "grafana",
        name: "Grafana",
        summary: "Paineis e graficos sobre os dados. A IDE conversa com ele por HTTP API — \
                  nunca embutido, porque o Grafana e' AGPL.",
        website: "https://grafana.com/docs/grafana/latest/setup-grafana/installation/",
        probe_binary: "grafana",
    },
];

// --------------------------------------------------------------------------
// `PostgreSQL`
// Fonte: postgresql.org/download/linux/{redhat,ubuntu} — conferido 2026-09-04.
// --------------------------------------------------------------------------

static PG_REDHAT: &[Step] = &[
    Step {
        explanation: "Instala o servidor do PostgreSQL.",
        command: "sudo dnf install -y postgresql-server postgresql-contrib",
    },
    Step {
        explanation: "Cria o diretorio de dados do banco (so' na primeira vez).",
        command: "sudo postgresql-setup --initdb",
    },
    Step {
        explanation: "Sobe o servico e faz ele subir junto com a maquina.",
        command: "sudo systemctl enable --now postgresql.service",
    },
];

static PG_DEBIAN: &[Step] = &[Step {
    explanation: "Instala o PostgreSQL que vem na sua distribuicao.",
    command: "sudo apt install -y postgresql",
}];

// --------------------------------------------------------------------------
// `TimescaleDB`
// Fonte: tigerdata.com/docs/self-hosted/latest/install/installation-linux
// Conferido 2026-09-04. A propria fonte instala o `PostgreSQL` junto.
// --------------------------------------------------------------------------

static TS_DEBIAN: &[Step] = &[
    Step {
        explanation: "Ferramentas que o repositorio precisa para ser configurado.",
        command: "sudo apt install -y gnupg postgresql-common apt-transport-https lsb-release wget",
    },
    Step {
        explanation: "Configura o repositorio oficial do PostgreSQL.",
        command: "sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh",
    },
    Step {
        explanation: "Acrescenta o repositorio do TimescaleDB.",
        command: "echo \"deb https://packagecloud.io/timescale/timescaledb/debian/ \
                  $(lsb_release -c -s) main\" | sudo tee /etc/apt/sources.list.d/timescaledb.list",
    },
    Step {
        explanation: "Instala a chave que assina esse repositorio.",
        command: "wget --quiet -O - https://packagecloud.io/timescale/timescaledb/gpgkey \
                  | sudo gpg --dearmor -o /etc/apt/trusted.gpg.d/timescaledb.gpg",
    },
    Step {
        explanation: "Atualiza a lista de pacotes e instala a extensao.",
        command: "sudo apt update && sudo apt install -y timescaledb-2-postgresql-18 \
                  postgresql-client-18",
    },
    Step {
        explanation: "Ajusta a configuracao do PostgreSQL para series temporais e reinicia.",
        command: "sudo timescaledb-tune && sudo systemctl restart postgresql",
    },
];

static TS_REDHAT: &[Step] = &[
    Step {
        explanation: "Acrescenta o repositorio oficial do `PostgreSQL` (o PGDG).",
        command: "sudo dnf install -y \
                  https://download.postgresql.org/pub/repos/yum/reporpms/EL-$(rpm -E %{rhel})-x86_64/pgdg-redhat-repo-latest.noarch.rpm",
    },
    Step {
        explanation: "Instala a extensao e o `PostgreSQL` correspondente.",
        command: "sudo dnf install -y timescaledb-2-postgresql-18 postgresql18",
    },
    Step {
        explanation: "Cria o diretorio de dados do banco.",
        command: "sudo /usr/pgsql-18/bin/postgresql-18-setup initdb",
    },
    Step {
        explanation: "Ajusta a configuracao para series temporais e sobe o servico.",
        command: "sudo timescaledb-tune --pg-config=/usr/pgsql-18/bin/pg_config \
                  && sudo systemctl enable --now postgresql-18",
    },
];

// --------------------------------------------------------------------------
// Grafana
// Fonte: grafana.com/docs/grafana/latest/setup-grafana/installation/{debian,
// redhat-rhel-fedora} — conferido 2026-09-04.
// --------------------------------------------------------------------------

static GRAFANA_DEBIAN: &[Step] = &[
    Step {
        explanation: "Ferramentas que o repositorio precisa.",
        command: "sudo apt-get install -y apt-transport-https wget gnupg",
    },
    Step {
        explanation: "Cria o diretorio das chaves e baixa a chave oficial do Grafana.",
        command: "sudo mkdir -p /etc/apt/keyrings && sudo wget -O \
                  /etc/apt/keyrings/grafana.asc https://apt.grafana.com/gpg-full.key",
    },
    Step {
        explanation: "Acrescenta o repositorio estavel do Grafana.",
        command: "echo \"deb [signed-by=/etc/apt/keyrings/grafana.asc] \
                  https://apt.grafana.com stable main\" | sudo tee -a \
                  /etc/apt/sources.list.d/grafana.list",
    },
    Step {
        explanation: "Atualiza a lista de pacotes e instala a edicao OSS.",
        command: "sudo apt-get update && sudo apt-get install -y grafana",
    },
];

static GRAFANA_REDHAT: &[Step] = &[Step {
    explanation: "Instala a edicao OSS a partir do repositorio RPM do Grafana.",
    command: "sudo dnf install -y grafana",
}];

// --------------------------------------------------------------------------
// Python: pipx, uv, ruff, basedpyright — conferido 2026-09-12.
// pipx: pipx.pypa.io/stable/how-to/install-pipx.html (Fedora e Ubuntu 23.04+
// documentados; "outras distros" via pip --user, que a PEP 668 bloqueia nas
// distros novas — por isso nao entra como guia).
// uv: docs.astral.sh/uv/getting-started/installation (pipx, pip, curl; nenhuma
// distro documentada). ruff: docs.astral.sh/ruff/installation (pipx, uv tool,
// curl; pacman no Arch; zypper no openSUSE). basedpyright:
// docs.basedpyright.com (uv tool install, pip, npm; sem pipx documentado).
// --------------------------------------------------------------------------

static PIPX_REDHAT: &[Step] = &[
    Step {
        explanation: "Instala o pipx pelo gerenciador da distro (a PEP 668 bloqueia o pip --user).",
        command: "sudo dnf install pipx",
    },
    Step {
        explanation: "Poe ~/.local/bin no PATH, onde o pipx instala as ferramentas.",
        command: "pipx ensurepath",
    },
];

static PIPX_DEBIAN: &[Step] = &[
    Step {
        explanation: "Atualiza a lista de pacotes.",
        command: "sudo apt update",
    },
    Step {
        explanation: "Instala o pipx pelo gerenciador da distro (Ubuntu 23.04 ou mais novo).",
        command: "sudo apt install pipx",
    },
    Step {
        explanation: "Poe ~/.local/bin no PATH, onde o pipx instala as ferramentas.",
        command: "pipx ensurepath",
    },
];

static UV_ANY: &[Step] = &[Step {
    explanation: "Instala o uv para o seu usuario, isolado, via pipx (a fonte tambem oferece \
                  `curl -LsSf https://astral.sh/uv/install.sh | sh`).",
    command: "pipx install uv",
}];

static RUFF_ANY: &[Step] = &[Step {
    explanation: "Instala o ruff para o seu usuario, isolado, via pipx (a fonte tambem oferece \
                  `uv tool install ruff@latest`).",
    command: "pipx install ruff",
}];

static RUFF_ARCH: &[Step] = &[Step {
    explanation: "Instala o ruff pelo gerenciador da distro, como a fonte escreve.",
    command: "pacman -S ruff",
}];

static RUFF_SUSE: &[Step] = &[Step {
    explanation: "Instala o ruff pelo gerenciador da distro (openSUSE Tumbleweed).",
    command: "sudo zypper install python3-ruff",
}];

static BASEDPYRIGHT_ANY: &[Step] = &[Step {
    explanation: "Instala o basedpyright para o seu usuario via uv (precisa do uv; a fonte \
                  tambem oferece `pip install basedpyright`).",
    command: "uv tool install basedpyright",
}];

// Bear — github.com/rizsotto/Bear (README conferido 2026-09-17: "bear --
// <your-build-command>"; instalacao "pelo gerenciador da distro"; as paginas
// de pacote responderam 200 nas tres familias).
static BEAR_DEBIAN: &[Step] = &[Step {
    explanation: "packages.debian.org/trixie/bear (Ubuntu 26.04: bear 3.1.6, medido).",
    command: "sudo apt install bear",
}];
static BEAR_REDHAT: &[Step] = &[Step {
    explanation: "packages.fedoraproject.org/pkgs/bear",
    command: "sudo dnf install bear",
}];
static BEAR_ARCH: &[Step] = &[Step {
    explanation: "archlinux.org/packages/extra/x86_64/bear",
    command: "sudo pacman -S bear",
}];

pub(super) static GUIDES: &[Guide] = &[
    Guide {
        tool: "pipx",
        family: "redhat",
        steps: PIPX_REDHAT,
        source_url: "https://pipx.pypa.io/stable/how-to/install-pipx.html",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "pipx",
        family: "debian",
        steps: PIPX_DEBIAN,
        source_url: "https://pipx.pypa.io/stable/how-to/install-pipx.html",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "uv",
        family: "any",
        steps: UV_ANY,
        source_url: "https://docs.astral.sh/uv/getting-started/installation/",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "ruff",
        family: "any",
        steps: RUFF_ANY,
        source_url: "https://docs.astral.sh/ruff/installation/",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "ruff",
        family: "arch",
        steps: RUFF_ARCH,
        source_url: "https://docs.astral.sh/ruff/installation/",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "ruff",
        family: "suse",
        steps: RUFF_SUSE,
        source_url: "https://docs.astral.sh/ruff/installation/",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "basedpyright",
        family: "any",
        steps: BASEDPYRIGHT_ANY,
        source_url: "https://docs.basedpyright.com/latest/installation/command-line-and-language-server/",
        checked_at: "2026-09-12",
    },
    Guide {
        tool: "postgresql",
        family: "redhat",
        steps: PG_REDHAT,
        source_url: "https://www.postgresql.org/download/linux/redhat/",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "postgresql",
        family: "debian",
        steps: PG_DEBIAN,
        source_url: "https://www.postgresql.org/download/linux/ubuntu/",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "timescaledb",
        family: "debian",
        steps: TS_DEBIAN,
        source_url: "https://www.tigerdata.com/docs/self-hosted/latest/install/installation-linux",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "timescaledb",
        family: "redhat",
        steps: TS_REDHAT,
        source_url: "https://www.tigerdata.com/docs/self-hosted/latest/install/installation-linux",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "grafana",
        family: "debian",
        steps: GRAFANA_DEBIAN,
        source_url: "https://grafana.com/docs/grafana/latest/setup-grafana/installation/debian/",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "grafana",
        family: "redhat",
        steps: GRAFANA_REDHAT,
        source_url: "https://grafana.com/docs/grafana/latest/setup-grafana/installation/redhat-rhel-fedora/",
        checked_at: "2026-09-04",
    },
    Guide {
        tool: "bear",
        family: "debian",
        steps: BEAR_DEBIAN,
        source_url: "https://packages.debian.org/trixie/bear",
        checked_at: "2026-09-17",
    },
    Guide {
        tool: "bear",
        family: "redhat",
        steps: BEAR_REDHAT,
        source_url: "https://packages.fedoraproject.org/pkgs/bear/bear/",
        checked_at: "2026-09-17",
    },
    Guide {
        tool: "bear",
        family: "arch",
        steps: BEAR_ARCH,
        source_url: "https://archlinux.org/packages/extra/x86_64/bear/",
        checked_at: "2026-09-17",
    },
];
