1.times do
  foo = ''
end

1.times do
  foo
  bar
end

def foo; end

puts(foo)
foo
foo()
puts(foo())

class MyTestClass
  it 'does something' do
    path = '/path/to/thing'
    File.write(path, 'Show me big bufo')
  end

  path = ''
  def path; end

  it 'does something else' do
    File.write(path, 'Show me small bufo, please')
    File.write(path(), 'One medium-sized bufo, if I may')
  end
end
