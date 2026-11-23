// Corpus of gate.duplication: correct form c14, the other parse.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include <cstddef>
#include <cstdint>
#include <optional>
#include <string_view>

namespace corpus::protocol {

std::optional<std::uint32_t> parse(std::string_view header) {
    constexpr std::string_view prefix{"Content-Length: "};
    if (!header.starts_with(prefix)) {
        return std::nullopt;
    }
    std::uint32_t length{0};
    for (const char digit : header.substr(prefix.size())) {
        if (digit == '\r') {
            break;
        }
        if (digit < '0' || digit > '9') {
            return std::nullopt;
        }
        length = length * 10U + static_cast<std::uint32_t>(digit - '0');
    }
    return length;
}

} // namespace corpus::protocol
