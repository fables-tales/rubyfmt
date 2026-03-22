-> (local) {
  cached_connections = applications.map { |x| x }
  return cached_connections.size == applications.size
  # for now assume cache miss
}

-> {
  y = x + 1
  y * 2
  # trailing comment
}
