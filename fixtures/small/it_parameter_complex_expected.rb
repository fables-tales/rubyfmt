result = [1, 2, 3, 4, 5]
  .select { it.even? }
  .map { it * 10 }
  .reduce(0) { |sum, n| sum + n }
