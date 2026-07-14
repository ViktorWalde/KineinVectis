import QtQuick

Item {
    id: root

    anchors.fill: parent
    enabled: false
    visible: TooltipController.shown && TooltipController.text !== ""

    KvTooltip {
        id: tooltip

        text: TooltipController.text
        x: {
            const preferred = TooltipController.placement === "right"
                    ? TooltipController.anchorX + TooltipController.anchorWidth
                      + Theme.spacingSmall
                    : TooltipController.anchorX
                      + (TooltipController.anchorWidth - width) / 2;
            return Math.max(Theme.spacingSmall,
                            Math.min(preferred,
                                     root.width - width - Theme.spacingSmall));
        }
        y: {
            const preferred = TooltipController.placement === "right"
                    ? TooltipController.anchorY
                      + (TooltipController.anchorHeight - height) / 2
                    : TooltipController.anchorY + TooltipController.anchorHeight
                      + Theme.spacingXSmall;
            return Math.max(Theme.spacingSmall,
                            Math.min(preferred,
                                     root.height - height - Theme.spacingSmall));
        }
    }
}
