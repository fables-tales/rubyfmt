#!/usr/bin/env ruby
require "delegate"
require "ripper"
require "stringio"

class Array
  def split(&blk)
    chunk(&blk).reject { |sep, _| sep }.map(&:last)
  end
end
