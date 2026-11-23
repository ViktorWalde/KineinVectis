// Corpus do gate de duplicação (P20), forma correta c05: três funções
// geradas por uma macro. A fonte é uma só. Cada função tem o tamanho que os
// dois detectores acusariam se a macro não ficasse de fora.
#include "corpus/protocol/messages.hpp"

#include <cstddef>
#include <span>

namespace corpus::protocol {

#define CORPUS_DEFINE_METHOD_COUNTER(function_name, wanted)                          \
    std::size_t function_name(std::span<const Method> methods, std::size_t limit) { \
        std::size_t matches{0};                                                     \
        std::size_t seen{0};                                                        \
        for (const Method method : methods) {                                       \
            if (seen == limit) {                                                    \
                break;                                                              \
            }                                                                       \
            ++seen;                                                                 \
            if (method == Method::wanted) {                                         \
                ++matches;                                                          \
            } else if (method == Method::shutdown) {                                \
                matches = 0;                                                        \
            }                                                                       \
        }                                                                           \
        return matches;                                                             \
    }

CORPUS_DEFINE_METHOD_COUNTER(count_initialize, initialize)
CORPUS_DEFINE_METHOD_COUNTER(count_open, open_document)
CORPUS_DEFINE_METHOD_COUNTER(count_close, close_document)

#undef CORPUS_DEFINE_METHOD_COUNTER

}  // namespace corpus::protocol
