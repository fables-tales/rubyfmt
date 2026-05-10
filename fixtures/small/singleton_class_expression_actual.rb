class << self
  def foo; end
end

class << receiver
  def foo; :bar; end
end

class << @ivar
  def foo; end
end

class << @@cvar
  def foo; end
end

class << $global
  def foo; end
end

class << Foo
  def foo; end
end

class << SomeModule::CONST
  def foo; end
end

class << foo.bar
  def foo; end
end
