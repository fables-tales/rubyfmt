it '' do
  Test::Mock.expects(Util, :download_file)
    .twice # Should only download the patches twice
    .returns(true)
end
  
# Make some fake thing
Test::FakeData.make_name_and_id(
  person: person,
  status: ActionStatus::Match,
  created: now + 5
) # Made the thing!
  
RELATIVE_EXCLUDES.any? { |str| things.include?(str) } ||
# We've got all these sorts of reasons
# we need to filter these out, but
# someone will probably document that elsewhere, not here
!relative.end_with?('.rb')
