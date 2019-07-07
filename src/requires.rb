#!/usr/bin/env ruby
require "delegate"
require "ripper"
require "stringio"

MAX_WIDTH = 100
GC.disable

class Array
  def split(&blk)
    chunk(&blk).reject { |sep, _| sep }.map(&:last)
  end

  def rindex_by(&blk)
    reverse_each.each_with_index do |item, idx|
      if blk.call(item)
        return length - idx
      end
    end

    nil
  end
end
