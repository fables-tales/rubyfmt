def example
  -> (arg_a, arg_b) {
    # On multiple lines!
    with_comments!
    # whew this is so cool that this works
  }

  ->() {
    this_one_has
    multiple_expressions!
  }

  ->() do { this_one_is: "weird" } end

  ->(arg) { ok_finally_a_normal_one }
end
