# frozen_string_literal: true
# $Id: test_dryrun.rb 62738 2018-03-13 06:29:02Z nobu $

require 'fileutils'
require 'test/unit'
require_relative 'visibility_tests'

class TestFileUtilsDryRun < Test::Unit::TestCase

  include FileUtils::DryRun
  include TestFileUtilsInc::Visibility

  def setup
    super
    @fu_module = FileUtils::DryRun
  end

end
