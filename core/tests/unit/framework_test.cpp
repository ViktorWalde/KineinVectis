// O GoogleTest foi compilado no mesmo modo da biblioteca padrao do teste.
//
// No perfil debug-stl, o _GLIBCXX_DEBUG troca std::vector por
// std::__debug::vector, e o tipo de PARAMETRO entra no nome mangled da funcao
// (o de retorno nao entra). A CalculateOptimalEdits mora na biblioteca do
// GoogleTest e recebe std::vector: se a biblioteca tiver sido compilada sem o
// modo de depuracao, o simbolo pedido aqui nao existe nela, e o teste nao
// liga. E' isso que ele prova; nao testa codigo do core.
#include <gtest/gtest.h>

#include <cstddef>
#include <vector>

namespace edit_distance = testing::internal::edit_distance;

TEST(TestFramework, SharesTheStandardLibraryModeWithTheTests)
{
    const std::vector<std::size_t> left{1, 2};
    const std::vector<std::size_t> right{1, 3};
    const std::vector<edit_distance::EditType> expected{edit_distance::kMatch,
                                                        edit_distance::kReplace};
    EXPECT_EQ(edit_distance::CalculateOptimalEdits(left, right), expected);
}
