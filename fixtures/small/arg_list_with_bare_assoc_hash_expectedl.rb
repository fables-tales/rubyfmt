begin
  value = Integer(argument)
rescue ArgumentError
  RSpec.warning "Expected an integer value for `--fail-fast`, got: #{argument.inspect}", :call_site => nil
end
