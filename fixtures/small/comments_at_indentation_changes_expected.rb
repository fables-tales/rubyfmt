it "" do
  # Should only download the patches twice
  Test::Mock.expects(Util, :download_file).twice.returns(true)
end

# Make some fake thing
# Made the thing!
Test::FakeData.make_name_and_id(
  person: person,
  status: ActionStatus::Match,
  created: now + 5
)

# We've got all these sorts of reasons
# we need to filter these out, but
# someone will probably document that elsewhere, not here
RELATIVE_EXCLUDES.any? { |str| things.include?(str) } || !relative.end_with?(".rb")
