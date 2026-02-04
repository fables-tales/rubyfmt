if another_really_long_condition_that_makes_this_line_exceed_one_hundred_twenty_chars
  some_really_long_method_name_that_takes_up_space!
end

unless another_really_long_condition_that_makes_this_line_exceed_one_hundred_twenty
  some_really_long_method_name_that_takes_up_space!
end

short_method_call if a_condition_that_fits_within_the_line_length_limit_of_one_hundred_twenty

module X
  class Y
    module More
      def test
        # comments make a difference?
        amount = amount_from_api ||
          (
            if !line_items.nil?
              Opus::MonetaryFlows::Private::API::BeesItems::ParamTransformer.validation_amount(line_items)
            end
          )
      end
    end
  end
end
