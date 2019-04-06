def format_do_block(ps, rest)
  raise "got bad block #{rest.inspect}" if rest.length != 2
  params, body = rest

  ps.emit_do

  format_params(ps, params, " |", "|")

  ps.emit_newline

  ps.new_block do
    # in ruby 2.5 blocks are bodystmts because blocks support
    # ```
    # foo do
    # rescue
    # end
    # ```
    #
    # style rescues now
    if body[0] == :bodystmt
      format_expression(ps, body)
    else
      body.each do |expr|
        format_expression(ps, expr)
      end
    end
  end

  ps.with_start_of_line(true) do
    ps.emit_end
  end
end
