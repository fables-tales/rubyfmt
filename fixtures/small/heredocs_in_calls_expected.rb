puts(
  1,
  2,
  <<~TXT
  3
  4
  TXT
)

puts(
  1,
  2,
  <<TXT
    3
    4
TXT
)

def foo
  puts(
    1,
    2,
    <<~TXT
    3
    4
    TXT
    )

  puts(
    1,
    2,
    <<TXT
      3
      4
TXT
    )
end

foo
