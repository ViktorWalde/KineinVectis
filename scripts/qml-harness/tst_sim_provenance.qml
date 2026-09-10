// A TELA DIZ DE ONDE VEIO O "VALOR EXATO".
//
// POR QUE ESTE HARNESS EXISTE (2026-09-10). Medido contra o binario real em
// 2026-09-06 (`roadmaps/31` §19.0): com `-(k/m)*x - 2*(c/m)*v` no oscilador
// amortecido a IDE acusava erro de 2,5e-2 numa integracao correta ate' 3,2e-7 —
// setenta e oito mil vezes o que reportava. A coluna `exato` vinha da solucao
// do CONCEITO e nao olhava a formula digitada; o checador aprova a formula
// porque confere ligacao, nao fisica.
//
// O conserto NAO foi esconder o numero — sem o oraculo, a solucao do conceito
// continua sendo a melhor resposta disponivel. O conserto foi **dizer de onde
// ele vem**, e este harness cobra exatamente isso na tela.
//
// O que ele cobra:
//   1. com o oraculo, a tela diz que resolveu a SUA equacao, e mostra a solucao
//   2. sem o oraculo, a RESSALVA aparece — o numero e' de outra pergunta
//   3. o erro RELATIVO aparece ao lado do absoluto
//   4. o veredito de UNIDADE aparece, com o LIMITE dito junto
//   5. sem a ferramenta, a tela diz que NAO conferiu
//
// MUTACOES QUE PROVAM O GATE:
//   - tire o `SimAccuracyProvenance` do `SimRunResultView`   -> falha 1 e 2
//   - troque `root.source === "oracle"` por `false`
//     no `SimAccuracyProvenance`                              -> falha 1
//   - tire a linha do erro relativo                           -> falha 4
//   - tire o `SimDimensionsView` do `SimRunResultView`         -> falha 256
//   - tire a linha do LIMITE ("unidade que fecha nao quer
//     dizer fisica certa")                                     -> falha 512
import QtQuick
import KineinVectis

Item {
    id: root

    width: 1000
    height: 900

    property int falhas: 0

    // O que o core devolve quando o oraculo respondeu.
    function comOraculo() {
        return {
            stepsTaken: 100,
            sampleEvery: 2,
            methodLabel: "Runge-Kutta 4, dt = 0,1",
            trail: [{ t: 0, y: 1, dy: 0 }, { t: 10, y: 0.032, dy: 0 }],
            accuracy: {
                exact: 0.0321283198320319,
                numeric: 0.0321320518,
                absoluteError: 3.732e-06,
                relativeError: 1.1616e-04,
                source: "oracle",
                solvedBy: "SymPy 1.14.0",
                closedForm: "(sqrt(31)*sin(sqrt(31)*t/4)/31 + cos(sqrt(31)*t/4))*exp(-t/4)"
            },
            dimensions: { equation: "", verdict: "wrongSide",
                          detail: "length*time^-2 != length^3*time^-2" }
        };
    }

    // E o que ele devolve quando nao ha' oraculo nesta maquina.
    // Sem a ferramenta NAO ha' veredito de unidade — e a tela tem de dizer.
    function semOraculo() {
        return {
            stepsTaken: 100,
            sampleEvery: 2,
            methodLabel: "Runge-Kutta 4, dt = 0,1",
            trail: [{ t: 0, y: 1, dy: 0 }, { t: 10, y: 0.032, dy: 0 }],
            accuracy: {
                exact: 0.0321283198320319,
                numeric: 0.0321320518,
                absoluteError: 3.732e-06,
                relativeError: 1.1616e-04,
                source: "concept"
            }
        };
    }

    readonly property string ressalva:
        "O valor exato vem da solução do conceito, não da equação que você escreveu: "
        + "para a IDE resolver a SUA equação ela precisa do SymPy."

    SimRunResultView {
        id: vista

        width: root.width - 40
        run: root.comOraculo()
        oracleNote: ""
    }

    function achar(item, tipo) {
        for (let i = 0; i < item.children.length; ++i) {
            const filho = item.children[i];
            if (String(filho).indexOf(tipo) === 0) {
                return filho;
            }
            const achado = root.achar(filho, tipo);
            if (achado !== null) {
                return achado;
            }
        }
        return null;
    }

    // Junta todo texto visivel abaixo de um item, para as assercoes falarem de
    // TELA e nao de propriedade interna.
    function textoVisivel(item) {
        let junto = "";
        if (!item.visible) {
            return junto;
        }
        if (item.text !== undefined && typeof item.text === "string") {
            junto += item.text + "\n";
        }
        for (let i = 0; i < item.children.length; ++i) {
            junto += root.textoVisivel(item.children[i]);
        }
        return junto;
    }

    Timer {
        interval: 400
        running: true
        repeat: false

        onTriggered: {
            const procedencia = root.achar(vista, "SimAccuracyProvenance");
            if (procedencia === null) {
                console.warn("a tela nao diz de onde veio o valor exato");
                root.falhas += 1;
                Qt.exit(1);
                return;
            }

            // 1. Com o oraculo: a tela nomeia quem resolveu e mostra a solucao.
            const comOraculo = root.textoVisivel(procedencia);
            if (comOraculo.indexOf("SymPy 1.14.0") < 0) {
                console.warn("a tela nao diz QUEM resolveu: " + comOraculo);
                root.falhas += 1;
            }
            if (comOraculo.indexOf("sqrt(31)") < 0) {
                console.warn("a resposta verdadeira nao aparece ao lado do calculado");
                root.falhas += 2;
            }

            // 3. O erro relativo, ao lado do absoluto.
            const tudo = root.textoVisivel(vista);
            if (tudo.indexOf("erro relativo") < 0) {
                console.warn("o erro relativo nao esta na tela: so' o absoluto engana");
                root.falhas += 4;
            }

            // 4. O veredito de UNIDADE, e o LIMITE dito junto com ele.
            const unidades = root.achar(vista, "SimDimensionsView");
            const textoUnidades = unidades === null ? "" : root.textoVisivel(unidades);
            if (textoUnidades.indexOf("não é da grandeza do lado esquerdo") < 0) {
                console.warn("o veredito de unidade nao esta na tela: " + textoUnidades);
                root.falhas += 256;
            }
            if (textoUnidades.indexOf("não quer dizer física certa") < 0) {
                console.warn("o LIMITE nao esta na tela — sem ele, 'as unidades fecham' "
                             + "vira promessa de que a fisica esta certa");
                root.falhas += 512;
            }

            vista.run = root.semOraculo();
            vista.oracleNote = root.ressalva;
            depois.restart();
        }
    }

    Timer {
        id: depois

        interval: 400
        repeat: false

        onTriggered: {
            const procedencia = root.achar(vista, "SimAccuracyProvenance");
            const semOraculo = procedencia === null ? "" : root.textoVisivel(procedencia);

            // 2. Sem o oraculo, a RESSALVA aparece — e ela e' o conserto.
            if (semOraculo.indexOf("não da equação que você escreveu") < 0) {
                console.warn("a ressalva de procedencia nao apareceu: " + semOraculo);
                root.falhas += 8;
            }
            if (semOraculo.indexOf("SymPy 1.14.0") >= 0) {
                console.warn("a tela diz que o oraculo resolveu quando ele nao resolveu");
                root.falhas += 16;
            }

            // 5. Sem a ferramenta, a tela DIZ que nao conferiu as unidades — em
            //    vez de parar de conferir em silencio.
            const unidades = root.achar(vista, "SimDimensionsView");
            const texto = unidades === null ? "" : root.textoVisivel(unidades);
            if (texto.indexOf("não foram verificadas nesta sessão") < 0) {
                console.warn("a tela nao diz que deixou de conferir as unidades: " + texto);
                root.falhas += 1024;
            }

            if (root.falhas !== 0) console.error("FALHAS bitmask=" + root.falhas);
            Qt.exit(root.falhas === 0 ? 0 : 1);
        }
    }
}
