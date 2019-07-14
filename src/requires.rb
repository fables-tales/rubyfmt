#!/usr/bin/env ruby
require "delegate"
require "ripper"
require "stringio"

MAX_WIDTH = 100

class Array
  def split(&blk)
    chunk(&blk).reject { |sep, _| sep }.map(&:last)
  end

  def rindex_by
    reverse_each.each_with_index do |item, idx|
      if yield item
        return length - idx
      end
    end

    nil
  end
end
