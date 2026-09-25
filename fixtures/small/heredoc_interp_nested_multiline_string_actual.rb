X = <<~H
  #{foo("a
    b")}
H

SQL = <<~SQL.squish
  MAX(#{foo("CASE WHEN a
    THEN b ELSE 0 END", "CASE WHEN c
    THEN d ELSE 0 END")})
SQL

<<~JS
  #{foo(
    bar: "aaa,
          bbb",
  )}
JS

Y = <<~H
  #{foo("a#{1}
    b")}
H

Z = <<-H
  #{foo("a
    b")}
H
