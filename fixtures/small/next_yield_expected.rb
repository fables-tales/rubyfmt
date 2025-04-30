def foo(&blk)
  [].each do
    next yield a
  end
end
