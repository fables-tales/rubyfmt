->(local) {
  cached_connections = applications.map {|x| x}
  return cached_connections.size == applications.size
  # for now assume cache miss
}

-> {
  y = x + 1
  y * 2
  # trailing comment
}

->(local) do
  cached_connections = applications.map {|x| x}
  return cached_connections.size == applications.size
  # for now assume cache miss
end

-> do
  y = x + 1
  y * 2
  # trailing comment
end
