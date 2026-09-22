# Implementação em Qt/QML

## 1. Recursos

Adicionar o módulo de glifos à unidade QML. As referências PNG grandes não são
compiladas:

```cmake
qt_add_qml_module(kinein-vectis
    # ...
    QML_FILES
        qml/components/KvIcon.qml
        qml/components/KvFileIcon.qml
        qml/components/KvFileIconGlyphs.js
)
```

## 2. Seleção de tamanho

```qml
pragma Singleton
import QtQuick

QtObject {
    readonly property int compactIconSize: 16
    readonly property int defaultIconSize: 20
    readonly property int comfortableIconSize: 24

    readonly property int compactRowHeight: 20
    readonly property int defaultRowHeight: 24
    readonly property int comfortableRowHeight: 28
}
```

## 3. Componente

```qml
import QtQuick
import "KvFileIconGlyphs.js" as FileGlyphs

Canvas {
    id: iconCanvas

    required property string iconName
    property int iconSize: 20

    width: iconSize
    height: iconSize

    onPaint: {
        const context = getContext("2d");
        context.reset();
        context.scale(width / 24, height / 24);
        FileGlyphs.draw(iconName, context);
    }
}
```

## 4. Delegate da árvore

```qml
Item {
    id: row

    required property var model
    property int iconSize: 20
    property int rowHeight: 24

    implicitHeight: rowHeight

    KFileIcon {
        id: fileIcon

        anchors.left: parent.left
        anchors.leftMargin: 4
        anchors.verticalCenter: parent.verticalCenter

        iconSize: row.iconSize
        iconName: model.iconAssetName
    }

    Text {
        anchors.left: fileIcon.right
        anchors.leftMargin: 6
        anchors.verticalCenter: parent.verticalCenter

        text: model.displayName
        color: Theme.treeText
        elide: Text.ElideRight
    }
}
```

## 5. Regras de nitidez

- manter `x`, `y`, `width` e `height` em valores inteiros;
- não aplicar `scale: 0.8` ou transformações fracionárias;
- usar a grade lógica 24x24 em 16, 20 ou 24 px inteiros;
- preservar traços fortes e uma única metáfora principal;
- solicitar nova pintura apenas quando nome, cor ou tamanho mudar;
- não carregar os PNGs originais de mais de 1200 px na árvore;
- não incluir microtexto, blur, brilho ou sombras.

## 6. Estratégia de desempenho

Para a primeira implementação:

```text
Canvas compartilhado + módulo JavaScript de geometria vetorial.
```

Quando a árvore tiver milhares de nós, medir:

```text
tempo até primeiro frame;
tempo de abertura de pasta;
uso de CPU durante scroll;
memória da cache;
jank de frame.
```

Se a pintura aparecer no perfil, medir primeiro o cache de cena e então avaliar
SVGs dedicados por família. Os originais grandes ficam como entrada de design e
não entram no recurso Qt.
