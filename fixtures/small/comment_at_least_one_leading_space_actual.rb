#! /usr/bin/env ruby
# frozen_string_literal: true

#### #### AUTHOR: coder204coolgizmos ### ##

##DATE: 2026-06-27
##   Cool Gizmos, Inc.

#### ####
###----------
#:
#
#
#This is a class that frobs a gizmo. It:
#  * frobs the sprog
#  * then turns the sprog into the gizmo.
#### #
#### #
#Keep this class here.
class Foo
  #inner class comment

  #method description comment
  def self.b(&_blk)
    yield
  end

  #method description comment
  def self.a
    #some method comment

    x = b do
      #some comment inside the block
      y = 4
      # some other comment
    end
  end
end
