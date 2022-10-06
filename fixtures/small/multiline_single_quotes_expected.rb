example = "a
  b
     c
 d
"

dj_khaled_another_one = "
  there's a bunch of stuff in here
  like newline characters(\\n) that don't
  \\n \\n actually become whitespace!\\n
"

query = "
  group {
    person {
      attribute {
        slug
      }
    }
    created
    updated
    otherPeople {
      name
    }
  }
"

string_with_slash = "
  example \
  someone wrote this for some reason
"
