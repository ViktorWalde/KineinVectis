import QtQuick

// O FILTRO DO RESULTADO QUE JA' CHEGOU (§5.2: "filtro atua localmente sobre o
// resultado ja' recebido").
//
// LOCAL de proposito, e isso e' decisao, nao economia: filtrar no servidor
// faria cada tecla virar uma chamada HTTP a um Grafana que pode estar do outro
// lado de uma VPN, e transformaria digitar num jeito de medir latencia. O que
// chegou, chegou; refinar o que se ve e' trabalho da tela.
//
// Sem regex, sem curinga: o texto e' procurado como texto. Um `*` digitado por
// engano nao pode virar sintaxe, e `.` e' comum demais em nome de host para
// significar "qualquer caractere".
QtObject {
    id: root

    // Os campos que uma linha oferece para busca, por lista. Estar aqui, e nao
    // espalhado nos delegates, e' o que faz "procurar por pasta" ser uma
    // frase verdadeira sobre dashboards e falsa sobre fontes de dados.
    readonly property var camposDashboard: ["title", "folderTitle", "url"]
    readonly property var camposFonte: ["name", "typeName", "typeId", "url"]

    function normalizar(texto) {
        return texto === undefined || texto === null ? "" : String(texto).toLowerCase();
    }

    // Uma linha casa se QUALQUER campo declarado contem o texto. E buscar por
    // "" devolve tudo: filtro vazio nao e' filtro.
    function rowMatches(linha, campos, busca) {
        const agulha = root.normalizar(busca).trim();
        if (agulha === "") {
            return true;
        }
        for (let indice = 0; indice < campos.length; ++indice) {
            if (root.normalizar(linha[campos[indice]]).indexOf(agulha) >= 0) {
                return true;
            }
        }
        return false;
    }

    function apply(linhas, campos, busca) {
        if (linhas === undefined || linhas === null) {
            return [];
        }
        const agulha = root.normalizar(busca).trim();
        if (agulha === "") {
            return linhas;
        }
        const saida = [];
        for (let indice = 0; indice < linhas.length; ++indice) {
            if (root.rowMatches(linhas[indice], campos, agulha)) {
                saida.push(linhas[indice]);
            }
        }
        return saida;
    }

    // "NENHUM RESULTADO" E' DIFERENTE DE "NAO HA' NADA", e a frase precisa
    // dizer qual dos dois e' — senao o autor sonda de novo procurando um
    // dashboard que esta' ali, escondido pelo proprio filtro.
    function emptyPhrase(totalRecebido, busca) {
        const agulha = root.normalizar(busca).trim();
        if (agulha !== "" && totalRecebido > 0) {
            return qsTr("nada casa com “%1” — apague o filtro para ver os %2")
                     .arg(busca.trim()).arg(totalRecebido);
        }
        return "";
    }
}
