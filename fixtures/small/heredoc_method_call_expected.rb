class William::Carlos::Williams
  landscape_with_the_fall_of_icarus = T.let(
    new(
      <<~LANDSCAPE
        According to Brueghel
        when Icarus fell
        it was spring

        a farmer was ploughing
        his field
        the whole pageantry

        of the year was
        awake tingling
        with itself

        sweating in the sun
        that melted
        the wings' wax

        unsignificantly
        off the coast
        there was

        a splash quite unnoticed
        this was
        Icarus drowning
      LANDSCAPE
    ),
    Williams
  )
end

optp
  .on do |value|
    <<~EOF
      There's some lines here

      But that one's a blank line!

      There shouldn't be any whitespace on those
    EOF
  end
