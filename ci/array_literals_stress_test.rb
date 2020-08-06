puts [].inspect
puts %w{1}.inspect
puts %W[].inspect
puts %i^^.inspect
puts ["word"].inspect
puts ['word'].inspect
puts ["word", 'word'].inspect
puts %w{word}.inspect
puts %W[word].inspect
puts %i{word}.inspect
puts %w{word}.inspect
puts %W[word].inspect
puts %w[word word].inspect
puts %W[word word].inspect
puts %w{word word}.inspect
puts %W{word word}.inspect
puts %i^word^.inspect
puts %I^[]^.inspect
puts %i^[^.inspect
puts %i^[#{1}]^.inspect
puts %I^[#{1}]^.inspect
puts %I^[]#{1}^.inspect
puts %w^[]^.inspect
puts %w^[^.inspect
puts %w^]^.inspect
puts %W^\^\[]^.inspect
puts %w^[#{1}]^.inspect
puts %W^[#{1}]^.inspect
puts %W^[]#{1}^.inspect
puts %W^\[^.inspect
puts %W^\^^.inspect
puts %w^\^^.inspect
puts %w[\]].inspect
puts %w(()).inspect
puts %w(\().inspect
puts %w(\)).inspect
puts %W(()).inspect
puts %W(\().inspect
puts %W(\)).inspect
puts %W(#{}()).inspect
puts %W(#{}\().inspect
puts %W(#{}\)).inspect
puts %W(()#{}).inspect
puts %W(\(#{}).inspect
puts %W(\)#{}).inspect
puts %w<\<>.inspect
puts %w<\>>.inspect
puts %W<<>>.inspect
puts %W<\<>.inspect
puts %W<\>>.inspect
puts %W<#{}<>>.inspect
puts %W<#{}\<>.inspect
puts %W<#{}\>>.inspect
puts %W<<>#{}>.inspect
puts %W<\<#{}>.inspect
puts %W<\>#{}>.inspect
puts %w{\{}.inspect
puts %w{\}}.inspect
puts %W{\}}.inspect
puts %W{\{}.inspect
puts %W{\}}.inspect
puts %W{#{}{}}.inspect
puts %W{#{}\{}.inspect
puts %W{#{}\}}.inspect
puts %W{{}#{}}.inspect
puts %W{\{#{}}.inspect
puts %W{\}#{}}.inspect
puts %W{#{%w(this [that)} [other]}.inspect
puts %W{#{%w[this \[that]} [other}.inspect
puts %W[#{%w(this [that)} \[other].inspect
puts %W{#{%w(this {that])} [other]}.inspect
puts %W{#{%w[this \[that]} [other}.inspect
puts %W[#{%w(this [that)} \[other].inspect
puts %w[\[].inspect
