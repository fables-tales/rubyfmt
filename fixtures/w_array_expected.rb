def func
  p(%w[a b c])
  p(%W[a b c #{Kernel.class.name}])
end

func
