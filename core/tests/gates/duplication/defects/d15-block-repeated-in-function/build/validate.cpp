// Corpus of gate.duplication: defect d15, the same block pasted twice inside the same function, for
// two arguments.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
#include "corpus/build/diagnostics.hpp"

#include <expected>
#include <string_view>

namespace corpus::build {

std::expected<std::size_t, ParseError> span_between(std::string_view first, std::string_view second) { // reject: span_between
    const auto first_fields = split_fields(first);
    if (!first_fields || first_fields->file.empty()) {
        return std::unexpected(ParseError::missing_location);
    }
    const auto first_line = parse_number(first_fields->line);
    if (!first_line || *first_line == 0) {
        return std::unexpected(ParseError::bad_line_number);
    }
    const auto second_fields = split_fields(second);
    if (!second_fields || second_fields->file.empty()) {
        return std::unexpected(ParseError::missing_location);
    }
    const auto second_line = parse_number(second_fields->line);
    if (!second_line || *second_line == 0) {
        return std::unexpected(ParseError::bad_line_number);
    }
    return *second_line > *first_line ? *second_line - *first_line : *first_line - *second_line;
}

} // namespace corpus::build
