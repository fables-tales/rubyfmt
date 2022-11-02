class Foo
  def bees
    begin
      return DRbRunner.new(options).run(err, out)
    rescue DRb::DRbConnError
      err.puts "No DRb server is running. Running in local process instead ..."
    else
      puts "Bees"
    end
  end
end

Foo.machine

begin
  # Doing stuff
  do_things!
  # Done stuff
rescue => e
  # Doing rescue things
  do_rescue_things!
  # Done doing rescue things
else
  # Doing other stuff
  do_other_stuff!
  # Done doing other stuff
end

# Some more comments for whatever reason