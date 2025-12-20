foo do |a, **more|
end

foo do |**more|
end

foo do |
  a, **more
  |
end

foo do |
    **more
  |
end