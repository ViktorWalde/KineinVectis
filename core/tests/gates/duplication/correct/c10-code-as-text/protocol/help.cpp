// Corpus of gate.duplication: correct form c10, code copied as text, in a literal and in a comment.
//
// Documented in: docs/engenharia/gates-e-medida-de-eficacia.md
//
// It is data, not logic.
#include <string_view>

namespace corpus::protocol {

// Example from the documentation:
//     std::vector<ByteOffset> starts{};
//     if (text.empty()) {
//         return starts;
//     }
//     starts.push_back(ByteOffset{0});
//     for (std::size_t index{0}; index < text.size(); ++index) {
//         const bool last = index + 1 == text.size();
//         if (text[index] == '\n' && !last) {
//             starts.push_back(ByteOffset{index + 1});
//         }
//     }
std::string_view wrap_example() {
    return R"(
    while (position < text.size()) {
        const auto start = text.find_first_not_of(' ', position);
        if (start == std::string_view::npos) {
            break;
        }
        auto end = text.find(' ', start);
        if (end == std::string_view::npos) {
            end = text.size();
        }
        const auto word = text.substr(start, end - start);
        current.append(word);
        position = end;
    })";
}

} // namespace corpus::protocol
