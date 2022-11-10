def main
  # I am going to a commune in Vermont and will
  # deal with no unit of time shorter than a season

  go_to_vermont!
rescue TrafficError
  puts("Had to deal with a unit of time shorter than a season")
  exit(1)
end
