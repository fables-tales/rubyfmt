loop do
  break no
  break
  break Some::Very::Long::Thing::That::Will::Extend::Over::Multiple::Lines::And::Never::Ends.await_result(
    workflow_id: workflow_id
  )
  break Some::Very::Long::Thing::That::Will::Extend::Over::Multiple::Lines::And::Never::Ends.await_result(
    workflow_id: workflow_id
  ), with_a_second_value
  break [
    off_reservation_payment,
    payment_intent_mode,
    Return.new(outcome: Outcome::Succeeded, attempts: off_reservation_payment.attempts + 1)
  ]
  break [
    off_reservation_payment,
    payment_intent_mode,
    Return.new(
      outcome: Outcome::Succeeded,
      attempts: off_reservation_payment.attempts + 1
    )
  ]
  break ([
    off_reservation_payment,
    payment_intent_mode,
    Return.new(outcome: Outcome::Succeeded, attempts: off_reservation_payment.attempts + 1)
  ])
end
