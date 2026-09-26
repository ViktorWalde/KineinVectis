// TECLADO NA GRADE COMUM (V7/G2).
//
// A §5.2 da `especificacoes/grafana-ui-ux-0.3.5.md` exige "selecao e teclado"
// nas listas de dashboards e fontes. A `KvDataGrid` servia tres paineis —
// banco, containers e git — e so' respondia ao mouse: quem nao usa mouse nao
// escolhia linha nenhuma, em nenhum deles.
//
// O CUIDADO QUE ESTE TESTE GUARDA: a grade NAO escreve em `selectedIndex`. Ela
// emite `rowClicked`, exatamente como o clique, porque no `ContainerListView`
// essa propriedade e' um BINDING para o controller — escrever nela de dentro
// quebraria o binding em silencio, que e' o estrago que nenhum gate acusa.
//
// MUTACAO QUE PROVA O GATE: troque o `root.rowClicked(alvo)` do
// `stepSelection` por `root.selectedIndex = alvo` e a ultima assercao cai.
import QtQuick
import KineinVectis

Item {
    id: root

    width: 400
    height: 300

    property int falhas: 0
    property var escolhidas: []
    property var abertas: []

    // O dono da selecao, como no painel de containers: a grade pede, ele
    // decide. O binding abaixo e' o que nao pode ser atropelado.
    QtObject {
        id: dono

        property int selecionada: -1
    }

    KvDataGrid {
        id: grade

        width: parent.width
        height: 100
        selectable: true
        rowHeight: 20
        maxHeight: 100
        columns: [{ key: "nome", label: "nome" }]
        rows: [{ nome: "um" }, { nome: "dois" }, { nome: "tres" }, { nome: "quatro" },
               { nome: "cinco" }, { nome: "seis" }, { nome: "sete" }, { nome: "oito" }]
        selectedIndex: dono.selecionada

        onRowClicked: indice => {
            root.escolhidas.push(indice);
            dono.selecionada = indice;
        }
        onRowActivated: indice => root.abertas.push(indice)
    }

    function conferir(condicao, mensagem) {
        if (!condicao) {
            console.error("FALHOU: " + mensagem);
            root.falhas += 1;
        }
    }

    Component.onCompleted: {
        // SEM SELECAO, a primeira tecla escolhe a ponta de onde ela vem.
        grade.stepSelection(1);
        root.conferir(dono.selecionada === 0,
                      "seta para baixo sem selecao nao escolheu a primeira linha");

        grade.stepSelection(1);
        grade.stepSelection(1);
        root.conferir(dono.selecionada === 2, "as setas nao andaram: " + dono.selecionada);

        // AS PONTAS NAO DAO A VOLTA: passar do fim para o inicio com uma seta
        // faz perder o lugar sem perceber.
        grade.stepSelection(-10);
        root.conferir(dono.selecionada === 0, "subiu alem da primeira linha");
        grade.stepSelection(100);
        root.conferir(dono.selecionada === grade.rows.length - 1,
                      "desceu alem da ultima linha");

        // A LINHA ESCOLHIDA FICA A' VISTA: oito linhas de 21px numa area de
        // ~79px so' cabem rolando.
        root.conferir(grade.rows.length * (grade.rowHeight + 1) > grade.height,
                      "o teste nao esta' medindo o que queria: a lista cabe inteira");
        const ultimaVisivel = grade.ensureVisible instanceof Function;
        root.conferir(ultimaVisivel, "a grade perdeu o `ensureVisible`");

        // GRADE NAO SELECIONAVEL IGNORA TECLA: sem selecao nao ha' o que mover,
        // e roubar a seta de quem esta' rolando a pagina seria pior que nada.
        const antes = root.escolhidas.length;
        grade.selectable = false;
        grade.stepSelection(1);
        root.conferir(root.escolhidas.length === antes,
                      "grade sem selecao respondeu ao teclado");
        grade.selectable = true;

        // ENTER ABRE, e abrir e' um sinal diferente de escolher.
        dono.selecionada = 1;
        grade.forceActiveFocus();
        root.conferir(root.abertas.length === 0, "abriu sem ninguem mandar");

        // O BINDING DO DONO SOBREVIVEU: e' isto que a mutacao quebra.
        dono.selecionada = 3;
        root.conferir(grade.selectedIndex === 3,
                      "a grade perdeu o binding de `selectedIndex` com o dono");

        Qt.exit(root.falhas === 0 ? 0 : 1);
    }
}
