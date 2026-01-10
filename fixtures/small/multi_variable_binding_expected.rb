def foo
  some, method = self.method()
  some, method = self.field.method
end

def bar
  (other, method) = self.method()
  (other, method) = self.field.method
end
