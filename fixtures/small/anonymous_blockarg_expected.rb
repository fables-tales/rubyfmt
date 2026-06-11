def foo(a, b, &)
  bar(a, b, &)
end

def foo(
  a,
  &
)
  bar(
    a,
    &
  )
end

def foo(
  a,
  *b,
  &
)
  bar(
    a,
    b,
    &
  )
end
