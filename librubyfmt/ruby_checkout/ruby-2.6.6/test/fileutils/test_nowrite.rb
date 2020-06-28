# frozen_string_literal: true
# $Id: test_nowrite.rb 62738 2018-03-13 06:29:02Z nobu $

require 'fileutils'
require 'test/unit'
require_relative 'visibility_tests'

class TestFileUtilsNoWrite < Test::Unit::TestCase

  include FileUtils::NoWrite
  include TestFileUtilsInc::Visibility

  def setup
    super
    @fu_module = FileUtils::NoWrite
  end

end
