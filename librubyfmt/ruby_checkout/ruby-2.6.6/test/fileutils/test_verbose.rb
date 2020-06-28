# frozen_string_literal: true
# $Id: test_verbose.rb 62738 2018-03-13 06:29:02Z nobu $

require 'test/unit'
require 'fileutils'
require_relative 'visibility_tests'

class TestFileUtilsVerbose < Test::Unit::TestCase

  include FileUtils::Verbose
  include TestFileUtilsInc::Visibility

  def setup
    super
    @fu_module = FileUtils::Verbose
  end

end
