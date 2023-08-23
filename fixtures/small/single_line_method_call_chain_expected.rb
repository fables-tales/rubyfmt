def blorp
  method_call(arg_a, arg_b).chained_call
end

This::Is::Some::SuperDuperLongConstantThatOnlyHasOneCallInTheChain.build(
  reason: Because::ThisCouldCauseUsSome::SeriousProblemsIfWeTriedToMultilineIt::EventThoughItActuallyDoesExtendBeyondTheLimit
)

This::Is::Some::SuperDuperLongConstantThatOnlyHasOneCallInTheChain
  .build(
    reason: Because::IPutThisOnMultipleLines
  )

this.surprisingly.can_break
# due to this comment being here!

# If there's a super long comment here that goes over the line limit weeeeeeeeeeeeeeeeeeee look at this one go
this.wont.break

# And there's a comment here
this
  .is_really_long
  .and_will_go_beyond_the_maximum_line_length_and_break
  .across_multiple_lines
  .and_it_has_a_comment_after_it!

this
  .is_really_long
  .and_will_go_beyond_the_maximum_line_length_and_break
  .across_multiple_lines
  .and_it_has_a_comment_after_it!
# See what happens to this comment here!
