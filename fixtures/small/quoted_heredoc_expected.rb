expected_result = <<-'EXPECTED'
One plus one is #{1 + 1}
EXPECTED

# prints: "One plus one is \#{1 + 1}\n"
p(expected_result)
