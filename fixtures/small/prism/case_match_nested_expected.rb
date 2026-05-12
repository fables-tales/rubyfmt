module App
  config = {db: {user: "admin", password: "abc123"}}

  case config
  in {db: {user:}}
    puts "Connect with user '#{user}'"
  in {connection: {username:}}
    puts "Connect with user '#{username}'"
  else
    puts "Unrecognized structure of config"
  end
end

class Foo
  def bar(value)
    case value
    in 1
      "one"
    in 2
      "two"
    end
  end
end
