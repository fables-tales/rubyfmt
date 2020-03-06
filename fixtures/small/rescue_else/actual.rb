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
