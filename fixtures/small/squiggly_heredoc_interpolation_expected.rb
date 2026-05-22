<<~FOO
  #{"foo"} bar
  more stuff
  stuff #{another_interpolation} things
  thing #{"#{interpolation} more"} even more
FOO

def foo
  <<~FOO
    #{"foo"} bar
    more stuff
    stuff #{another_interpolation} things
    thing #{"#{interpolation} more"} even more
  FOO
end

<<~EOF
  #{username} added this context to the operation:
   #{user_context}
EOF

<<-EOF
  #{username} added this context to the operation:
   #{user_context}
EOF

class Foo
  def to_chunks
    <<~JS
      #{array.map do |c|
        <<~JS
          "#{c[:path]}": () => import(/* webpackChunkName: '#{c[:name]}' */ '#{c[:path].gsub(%r{/index(/\.js)?\z}, "")}'),
        JS
      end}
    JS
  end
end
