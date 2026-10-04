import QtQuick
import KineinVectis

// A CAMADA FLUTUANTE, acima de tudo (z 10000 no Main): o tooltip e o menu de
// contexto dos campos de texto. Um lugar so' para o que tem de ficar por cima
// de qualquer dialogo — o seletor de pastas, as Configuracoes, o Banco.
Item {
    KvTextMenuHost {
        anchors.fill: parent
    }

    KvTooltipHost {
        anchors.fill: parent
    }
}
