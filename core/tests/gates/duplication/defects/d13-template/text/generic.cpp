// Corpus of gate.duplication: defect d13, line_starts copied as a template, instead of becoming a
// template in its owner.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
#include <cstddef>
#include <string_view>
#include <vector>

namespace corpus::text {

template <typename Char>
std::vector<std::size_t> line_starts_of(std::basic_string_view<Char> text) { // reject: line_starts
    std::vector<std::size_t> starts{};
    if (text.empty()) {
        return starts;
    }
    starts.push_back(0);
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool last = index + 1 == text.size();
        if (text[index] == Char{'\n'} && !last) {
            starts.push_back(index + 1);
        }
    }
    return starts;
}

std::size_t wide_lines(std::wstring_view text) {
    return line_starts_of(text).size();
}

} // namespace corpus::text
