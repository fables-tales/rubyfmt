<<~FOO
#{"foo"} bar
more stuff
stuff #{another_interpolation} things
thing #{"#{interploation} more"} even more
FOO

def foo
  <<~FOO
  #{"foo"} bar
  more stuff
  stuff #{another_interpolation} things
  thing #{"#{interploation} more"} even more
  FOO
end
