import QtQuick

// As PARTES da GitChangesList que o ListView instancia sozinho (o cabecalho de
// cada secao). Este arquivo NAO tem `pragma ComponentBehavior: Bound`, e isso
// e' o motivo de ele existir: no Qt 6.4 do AppImage (Debian 12), um
// `section.delegate`/`header`/`footer`/`highlight`/`sourceComponent` declarado
// num arquivo Bound NUNCA e' criado — cada secao virava "QQmlComponent:
// Component is not ready" e os cabecalhos de pasta do Git sumiam (provado por
// caso minimo, roadmap 53 §5.2.1). O tipo da secao acha a lista por
// `ListView.view`, sem id de fora.
QtObject {
    readonly property Component section: Component {
        GitChangesSection {}
    }
}
