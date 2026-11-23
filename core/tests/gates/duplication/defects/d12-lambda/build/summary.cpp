// Corpus of gate.duplication: defect d12, the body of line_starts copied into a lambda.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include "corpus/text/lines.hpp"

#include <cstddef>
#include <span>
#include <string>
#include <string_view>
#include <vector>

namespace corpus::build {

std::size_t count_breaks(std::span<const std::string> messages) {
    const auto offsets = [](std::string_view text) { // reject: line_starts
        std::vector<text::ByteOffset> starts{};
        if (text.empty()) {
            return starts;
        }
        starts.push_back(text::ByteOffset{0});
        for (std::size_t index{0}; index < text.size(); ++index) {
            const bool last = index + 1 == text.size();
            if (text[index] == '\n' && !last) {
                starts.push_back(text::ByteOffset{index + 1});
            }
        }
        return starts;
    };
    std::size_t total{0};
    for (const auto& message : messages) {
        total += offsets(message).size();
    }
    return total;
}

} // namespace corpus::build
