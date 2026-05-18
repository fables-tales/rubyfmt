# Call nodes don't need to disambiguate blocks
Case
Case()
Case { }
Case { }
Case do
end

Case do
end

Case(args) { }
Case(args) do
end

# But ConstantReadNodes must disambiguate blocks
Test::Case
Test::Case()
Test::Case() { }
Test::Case() { }
Test::Case() do
end

Test::Case(args) { }
Test::Case(args) do
end

Test.Case()
Test.Case()
Test.Case() { }
Test.Case() { }
Test.Case() do
end

Test.Case(args) { }
Test.Case(args) do
end

Case(&blk)
Case(args, &blk)
Test::Case(&blk)
Test::Case(args, &blk)
Test.Case(&blk)
Test.Case(args, &blk)
