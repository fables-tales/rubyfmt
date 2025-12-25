it "calls the function" do
  # temporarily set this
  ENV["CALL"] = "the function"
  the_function.call
ensure
  # reset
  ENV["CALL"] = nil
  # maybe something rubocop disable
end

it "calls the function again" do
  ENV["CALL"] = "the function again"
  the_function.call
ensure
  # reset
  ENV["CALL"] = nil
  # maybe something rubocop disable
end
