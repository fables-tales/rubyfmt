def boom
  raise("Ahhh!")
end

raise("plain")

x = raise("value")

raise(ArgumentError, "message")

raise("modifier") if condition

raise("unless") unless condition

def with_default(d = raise("y"))
end

def endless = raise("endless")

def keyword_default(k: raise("k"))
end

foo(raise("sole argument"))

bar(raise("nested sole argument"))

Foo.raise("receiver")

RSpec.describe("bees")

[foo(raise("nested sole argument"))]

{a: bar(raise("nested sole argument"))}

def nested(x = baz(raise("nested sole argument")))
end

[foo do
  raise("nested block body")
end]

[-> { raise("nested lambda body") }]

{a: proc do
  raise("nested hash block body")
end}
