import QtQuick
import KineinVectis

// A barra de status por PRIORIDADE (0.3.6, roadmap 53 §4.4; decisao do autor
// de 2026-10-01). A 1024 px o contexto do compilador saia cortado ao meio
// ("simbolos ‹"). O que se prova: os resumos entram inteiros, na ordem
// contexto > indice, conforme a largura que sobra; sem largura, nenhum; e a
// regra pura nao conta item vazio. (O Python saiu do rodape na 0.3.8 F3:
// mora no chip do cabecalho.)
Item {
    id: root

    width: 1200
    height: 40

    StatusBarProjectSummaries {
        id: summaries

        indexSummary: "6 arquivos · 16 linhas · 2 símbolos"
        contextSummary: "c++ · 0 -I · 0 -D"
    }

    function check(condition, message) {
        if (!condition) {
            console.error("FALHOU: " + message);
            return 1;
        }
        return 0;
    }

    function shown() {
        return [summaries.fitting[0], summaries.fitting[1]].join(",");
    }

    Component.onCompleted: {
        let failures = 0;
        const gap = summaries.spacing;

        // A regra pura.
        const f = summaries.fitByPriority;
        failures += check(f([50, 60, 70], 1000, 10).join(",") === "true,true,true", "tudo cabe");
        failures += check(f([50, 60, 70], 120, 10).join(",") === "true,true,false", "o ultimo sai");
        failures += check(f([50, 60, 70], 49, 10).join(",") === "false,false,false", "nada cabe");
        failures += check(f([0, 60, 70], 140, 10).join(",") === "false,true,true",
                          "item vazio nao ocupa nem espaco");
        // Um grande que nao cabe nao impede um menor depois dele.
        failures += check(f([200, 30], 100, 10).join(",") === "false,true", "menor depois do grande");

        // O componente: com espaco, os dois; apertando, sai o indice, depois
        // o contexto.
        summaries.availableWidth = 100000;
        failures += check(shown() === "true,true", "largo: " + shown());
        // Os filhos sao os textos, na ordem do arquivo: indice, contexto.
        const contextText = summaries.children[1];
        failures += check(contextText.text.indexOf("contexto:") === 0, "filho errado: " + contextText.text);
        const context = contextText.implicitWidth;
        summaries.availableWidth = context + gap + 5;
        failures += check(shown() === "true,false", "apertado: " + shown());
        summaries.availableWidth = context - 1;
        failures += check(!summaries.fitting[0], "sem espaco nem para o contexto: " + shown());

        // Sem contexto (nenhum arquivo C/C++ ativo), o indice passa a ser o primeiro.
        summaries.contextSummary = "";
        summaries.availableWidth = 100000;
        failures += check(shown() === "false,true", "sem contexto: " + shown());

        if (failures !== 0) console.error("FALHAS=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
