// Corpus do gate de duplicação (P20), defeito d06: line_starts devolvendo
// std::size_t no lugar de ByteOffset. Mesma lógica com outro tipo.
#include <cstddef>
#include <string_view>
#include <vector>

namespace corpus::text {

std::vector<std::size_t> break_positions(std::string_view text) {  // reprova: line_starts
    std::vector<std::size_t> starts{};
    if (text.empty()) {
        return starts;
    }
    starts.push_back(0);
    for (std::size_t index{0}; index < text.size(); ++index) {
        const bool last = index + 1 == text.size();
        if (text[index] == '\n' && !last) {
            starts.push_back(index + 1);
        }
    }
    return starts;
}

}  // namespace corpus::text
