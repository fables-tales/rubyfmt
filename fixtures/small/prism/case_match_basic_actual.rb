# Basic case/in pattern matching
case  value
in   1
    puts "one"
in    2
     puts "two"
else
   puts "other"
end

# Pattern matching without else
case  status
in   :ok
   puts "success"
in    :error
    puts "failure"
end
