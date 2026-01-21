def foo
  if a == b
    puts("bees")
  elsif "hi"
    "hi2"
  else
    puts("hi")
  end
end

do_stuff! if it_isnt_dangerous(
  i_promise: true
)

module Amp
  def self.trim_team(team)
    {
      descendent_members: (team[:descendant_members].map { |v| self.trim_person } if team[descendant_members]
        .present?)
    }
  end
end
