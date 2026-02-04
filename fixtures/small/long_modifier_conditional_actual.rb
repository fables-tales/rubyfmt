some_really_long_method_name_that_takes_up_space! if another_really_long_condition_that_makes_this_line_exceed_one_hundred_twenty_chars

some_really_long_method_name_that_takes_up_space! unless another_really_long_condition_that_makes_this_line_exceed_one_hundred_twenty

short_method_call if a_condition_that_fits_within_the_line_length_limit_of_one_hundred_twenty

module X
  class Y
    module More
      def test
        # comments make a difference?
        amount = amount_from_api ||
          (Opus::MonetaryFlows::Private::API::BeesItems::ParamTransformer.validation_amount(line_items) if !line_items.nil?)
      end
    end
  end
end
