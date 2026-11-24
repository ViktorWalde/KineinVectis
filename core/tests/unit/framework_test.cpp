// Test that GoogleTest was built in the same standard library mode as the tests.
//
// Documented in: docs/engineering/test-strategy.md
//
// In the debug-stl profile, _GLIBCXX_DEBUG replaces std::vector with
// std::__debug::vector, and the PARAMETER type enters the function's mangled
// name (the return type does not). CalculateOptimalEdits lives in the GoogleTest
// library and takes a std::vector: if the library was built without the debug
// mode, the symbol requested here does not exist in it, and the test does not
// link. That is what it proves; it does not test core code.
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
