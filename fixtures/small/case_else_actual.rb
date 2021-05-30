case
when Concurrent.on_jruby?
  JavaNonConcurrentPriorityQueue
else
  RubyNonConcurrentPriorityQueue
end
