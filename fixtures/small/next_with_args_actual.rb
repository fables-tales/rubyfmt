[].each do ||
next 1
next Foo.new(
    a,
    b
)
next(
    Foo.new(
        a,
        b
    )
)
end
