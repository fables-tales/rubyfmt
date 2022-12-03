# As above
a[0] = b
# So below

a[puts(a)] = ""
a[puts(a)] = ""
a[()] = ""
a[
  class A
    def foo
      puts(a)
    end
  end
] = ""
a[
  begin
    ""
  rescue StandardException
    ""
  end
] = ""
# Users can override #[]= to not have args
a[] = ""
# Users can override #[]= to have multiple args
a[1, 2] = ""

a[puts(a)]
a[puts(a)]
a[()]
a[
  class A
    def foo
      puts(a)
    end
  end
]
a[
  begin
    ""
  rescue StandardException
    ""
  end
]
# Users can override #[] to not have args
a[]
# Users can override #[] to have multiple args
a[1, 2]
