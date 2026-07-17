// Paleta e helper de formato do realce — compartilhados entre as regras regex
// (editor_highlighter_rules.cpp) e os formatos por token semantico/sintatico
// (editor_highlighter.cpp). Extraido em 2026-07-17 (Fase 1.4): as constantes de
// cor eram usadas por multiplos grupos do highlighter, entao viram header
// interno em vez de duplicar. Valores IDENTICOS aos do arquivo original.
#pragma once

#include <QColor>
#include <QFont>
#include <QTextCharFormat>

namespace kinein::highlighter_palette {

// Paleta de docs/05-design-system.md, em QRgb para inicializacao constexpr.
constexpr QRgb kKeywordRgb = 0xffffbb00; // accent
constexpr QRgb kStringRgb = 0xff7fbf7f;  // success soft
constexpr QRgb kCommentRgb = 0xff8f8a7c; // text muted
constexpr QRgb kNumberRgb = 0xff7aa2d8;  // info soft
constexpr QRgb kMetaRgb = 0xffd16d6d;    // error soft (macros, atributos, secoes)

// Find/Replace (D1b): fundo de TODAS as ocorrencias e da ocorrencia ATUAL.
// Ambar da paleta em duas intensidades — a atual "acende" sob o cursor.
constexpr QRgb kSearchMatchRgb = 0xff4a3a12;   // accent bem rebaixado
constexpr QRgb kSearchCurrentRgb = 0xff8a6a1a; // accent dim, ocorrencia atual

// Cores semanticas (LSP): distinguir variaveis, funcoes, tipos e campos.
constexpr QRgb kTypeRgb = 0xff5fb3ac;     // tipos/classes/enums/namespaces
constexpr QRgb kFunctionRgb = 0xffd8a657; // funcoes/metodos
constexpr QRgb kVariableRgb = 0xffcdd6e4; // variaveis
constexpr QRgb kPropertyRgb = 0xffb48ead; // campos/propriedades/enum members

inline QTextCharFormat colorFormat(QRgb rgb, bool bold = false, bool italic = false)
{
    QTextCharFormat format;
    format.setForeground(QColor::fromRgb(rgb));
    if (bold) {
        format.setFontWeight(QFont::DemiBold);
    }
    format.setFontItalic(italic);
    return format;
}

} // namespace kinein::highlighter_palette
