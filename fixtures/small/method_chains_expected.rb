returns_array
  .map { |foo|
    thing = foo.idk
    thing.call
  }
  .chain do |bar|
    bar.do_stuff!
  end
  .maybe_chain?(
    a: "",
    b: ""
  )
  .definitely_chain(
    a: "",
    b: ""
  )

foo.bar.baz

foo
  .bar
  .baz

foo
  .bar
  .baz

foo::bar
  &.nil?

foo::bar
  &.nil?::klass
  .true?

Class
  &.new
  .call!

def example
  things
    .map do |thing|
      case thing
      when Paul
        thing.call
      when Blart
        thing.also_call
      end
    end
    .uniq
end

foo.items.each { |item| item.call! }

foo.items.map { p(_1) }.each { _1.call! }

foo.items.map { p(_1) }.last

hashes.sort_by { |hsh| hsh[:start_time] }.reverse

params(
  route: String,
  config: T.nilable(Some::Really::Long::Type::Name),
  block: T.proc.bind(Some::Really::Long::Type::Name::In::This::Proc).void
)
  .void

Opus::Foo
  .params(
    route: String,
    config: T.nilable(Some::Really::Long::Type::Name),
    block: T.proc.bind(Some::Really::Long::Type::Name::In::This::Proc).void
  )
  .void

[
  "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "aaaaaaaaaaaaaaaaaAAAAAAAAAAAAAAAAAAAAAAAA",
  "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH"
].map do |shout|
  shout.upcase
end

foo.items.each do
  item.call!
end

foo
  .bar
  .baz
  .super_duper_extra_long_identifer
  .with_a_whole_bunch_of_other_really_long_things_in_here_to_make_super_duper_extra_sure
  .that_this_goes_past_one_hundred_and_twenty_characters_and_goes_to_separate_lines

paul = blart
  .bar
  .baz
  .super_duper_extra_long_identifer
  .with_a_whole_bunch_of_other_really_long_things_in_here_to_make_super_duper_extra_sure
  .that_this_goes_past_one_hundred_and_twenty_characters_and_goes_to_separate_lines

this_chain = is_long
  .but_really
  .only_the_actual_method_chain_part_is_long_but_we_should_still_account_for_that_when_we_consider_whether_or_not_multiline

MyModule::MyClass.foo(
  a: "",
  b: ""
)

var = MyModule::MyClass
  .foo
  .bar
  .baz
  .map { |x|
    multiline
    block
  }
  .bacon
  .next_call(
    a: "",
    b: ""
  )

ThisIs::OnlyOneCall
  # but it's explicitly multilined with a
  .comment!

OnlyOneCall
  # but it's explicitly multilined with a
  .comment!

[
  # foo
]
  # bar
  .baz

[]
  # Please don't do this
  .freeze

Paul::Blart::Mall::Cop::PerformedByTheLegendaryKevinJamesWhoIsAnAbsoluteLegendInAllOfHisFilmsWhichAreAbsolutelyIncredible
  .consume_pixie_sticks(mall: "downtown")
  .each do |punch_list_type|
  end

def gather_thanes!
  grendel = monsters
    .reject { |m| !alone?(m) }
    .filter { |m| ruiner_of_meadhalls?(m) }
    .reject { |m| (m < Wrecker::KingWrecker) }
    .reject { |m| (m < Wrangler::GoatWrangler) }
    .thing(
      a: "",
      b: ""
    )
    .fight_hrothgar
    # I discovered that the dragon had put a charm on me: no weapon could cut me.
    # I could walk up to the meadhall whenever I pleased, and they were powerless.
    .filter { |m| gold_can_be_added_to_pile?(m.gold) }

  grendel
end

x = [
  1,
  2
]
  .map do |x|
    # hi
    x.do_something
    # there
  end
  .flatten

x
  .hello_there
  &.map do |x|
    puts("h")
  end
  &.foo

My::Error.soft(
  "",
  stuff: {
    message_token: message.token,

    # Some comments!
    value: id_or_email.name
  }
)

# rubocop:disable PrisonGuard/PrivateModule
(foo
  .load_one
  # rubocop:enable PrisonGuard/PrivateModule
  .bar)
  .thing
