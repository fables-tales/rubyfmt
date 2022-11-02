<<~FOO
  Please don't #{
    puts("multiple expressions")
    inside = a_string_interpolation!

    # Man this is so confusing to read
    inside
  } it's so confusing, but so it goes.
FOO
