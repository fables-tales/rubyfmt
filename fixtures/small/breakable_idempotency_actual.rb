foo(-> { a; b })
foo(proc { a; b })
foo(lambda { a; b })
foo(bar { a; b })
[proc { a; b }, other]
{key: proc { a; b }}
foo([1, 2, 3], proc { a; b })
nested = foo(bar(baz { a; b }))
