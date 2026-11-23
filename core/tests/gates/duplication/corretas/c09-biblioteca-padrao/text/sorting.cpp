// Corpus do gate de duplicação (P20), forma correta c09: chamadas curtas à
// biblioteca padrão, parecidas e com outro efeito.
#include <algorithm>
#include <string>
#include <vector>

namespace corpus::text {

std::vector<std::string> sorted_unique(std::vector<std::string> words) {
    std::ranges::sort(words);
    const auto repeated = std::ranges::unique(words);
    words.erase(repeated.begin(), repeated.end());
    return words;
}

std::vector<std::string> sorted_by_length(std::vector<std::string> words) {
    std::ranges::stable_sort(words, {}, &std::string::size);
    return words;
}

}  // namespace corpus::text
