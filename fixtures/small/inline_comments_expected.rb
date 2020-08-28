module MathsAndPhysics
  # Used to model a record in DQT for the Maths and Physics policy.
  #
  # Should be initialised with data from a row in the report
  # requested from the Database of Qualified Teachers.
  #
  # Determines the eligibility of a teacher's qualifications for
  # the Maths and Physics policy.
  #
  #   qts_award_date:     The date the teacher achieved qualified
  #                       teacher status.
  #   itt_subject_codes:  The corresponding JAC codes to the subject
  #                       specialism that the teacher competed their
  #                       initial teacher training in.
  #   degree_codes:       The corresponding JAC codes to the subject(s)
  #                       the teacher completed their degree in.
  CONST = [
    #maths
    a,
    #physics
    b
  ]
end
