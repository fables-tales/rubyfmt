it "a" \
  "b" do
  #hi
end

it "a" do
  #hi
end

it "a", flag: true do
  #hi
end

it "a", flag: true, other: "b", another: false do
  #hi
end

describe "foo" do
  it "foo" do
  end
end

describe "foo", flag: true do
  it "foo" do
  end
end

describe "foo", flag: true, other: "b", another: false do
  it "foo" do
  end
end

RSpec.describe "bees" do
end

RSpec.describe "bees", flag: true do
end

RSpec.describe "bees", flag: true, other: "b", another: false do
end
