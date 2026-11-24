// Corpus of gate.duplication: correct form c08, the idiom of propagating the std::expected error,
// repeated with other calls and other errors.
//
// Documented in: docs/engineering/gates-and-effectiveness.md
//
// The shape repeats; the logic does not.
#include "corpus/build/diagnostics.hpp"

#include "corpus/text/lines.hpp"

#include <cstddef>
#include <expected>
#include <optional>
#include <string_view>

namespace corpus::build {

enum class SettingsError { missing_width, bad_width, missing_tab, bad_tab, too_narrow };

struct Layout {
    std::size_t width{};
    std::size_t tab{};
};

std::optional<std::string_view> value_of(std::string_view text, std::string_view key) {
    const auto at = text.find(key);
    if (at == std::string_view::npos) {
        return std::nullopt;
    }
    const auto end = text.find('\n', at);
    return text::trim(text.substr(at + key.size(), end == std::string_view::npos ? end : end - at - key.size()));
}

std::expected<Layout, SettingsError> load_layout(std::string_view text) {
    const auto width_text = value_of(text, "width=");
    if (!width_text) {
        return std::unexpected(SettingsError::missing_width);
    }
    const auto width = parse_number(*width_text);
    if (!width) {
        return std::unexpected(SettingsError::bad_width);
    }
    const auto tab_text = value_of(text, "tab=");
    if (!tab_text) {
        return std::unexpected(SettingsError::missing_tab);
    }
    const auto tab = parse_number(*tab_text);
    if (!tab) {
        return std::unexpected(SettingsError::bad_tab);
    }
    if (*width < 2 * *tab) {
        return std::unexpected(SettingsError::too_narrow);
    }
    return Layout{.width = *width, .tab = *tab};
}

} // namespace corpus::build
