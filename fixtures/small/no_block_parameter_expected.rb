def foo(&nil)
end

def bar(a, b, &nil)
end

foo do |a, &nil|
end

foo do |&nil|
end

foo do |
    a,
    &nil
  |
end

foo do |
    &nil
  |
end
