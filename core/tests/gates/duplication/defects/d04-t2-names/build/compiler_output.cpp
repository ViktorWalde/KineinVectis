// Corpus of gate.duplication: defect d04, T2, parse_diagnostic with other names, other formatting
// and comments.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/build/diagnostics.hpp"

#include <expected>
#include <string>
#include <string_view>

namespace corpus::build {

std::expected<Diagnostic, ParseError> read_compiler_line(std::string_view raw) { // reject: parse_diagnostic
    const auto parts = split_fields(raw);
    if (!parts) { return std::unexpected(ParseError::missing_location); }
    // The line and the column come from the compiler in decimal.
    const auto row = parse_number(parts->line);
    if (!row) { return std::unexpected(ParseError::bad_line_number); }
    const auto col = parse_number(parts->column);
    if (!col) { return std::unexpected(ParseError::bad_column); }
    const auto level = parse_severity(parts->severity);
    if (!level) { return std::unexpected(ParseError::unknown_severity); }
    return Diagnostic{.file = std::string{parts->file}, .line = *row, .column = *col,
                      .severity = *level, .message = std::string{parts->message}};
}

} // namespace corpus::build
