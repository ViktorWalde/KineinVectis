#!/usr/bin/env bash
# Hook carregado pelo AppRun gerado pelo linuxdeploy.
#
# O AppImage não pode pressupor que EGL/Wayland, Mesa ou um driver proprietário
# do host sejam compatíveis com o Qt do baseline. A UI atual é 2D e não usa
# ShaderEffect, portanto o renderer raster oficial do Qt Quick é o padrão mais
# previsível entre distribuições. A aceleração continua disponível por opt-in.

# Variáveis Qt explícitas sempre prevalecem. Sem override, o AppImage escolhe
# software para não acoplar a inicialização da IDE ao EGL/GL/Vulkan do host.
if [[ -z "${QT_QUICK_BACKEND+x}" && -z "${QSG_RHI_BACKEND+x}" ]]; then
    case "${KINEIN_GRAPHICS_BACKEND:-software}" in
        software | portable)
            export QT_QUICK_BACKEND=software
            ;;
        auto | hardware | gpu | native)
            ;;
        *)
            echo "aviso: KINEIN_GRAPHICS_BACKEND inválido; usando software" >&2
            export QT_QUICK_BACKEND=software
            ;;
    esac
fi
