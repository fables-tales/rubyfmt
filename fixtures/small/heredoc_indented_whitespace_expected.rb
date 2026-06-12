def foo
  <<~THING
    awfweaf 
      awefawef
      
      
      
    hi there
  THING

  <<-THING
  
  hi there
  THING
end

# whitespace-only lines are not considered when calculating indentation
puts(
  <<~FOO
    the next line contains a single space

  FOO
    .inspect
)

# 1 tab equals 8 spaces, both should be considered indentation
puts(
  <<~FOO
    this line is indented 8 spaces
    this line is indented 1 tab
  FOO
    .inspect
)

# 1 tab plus 1 space equals 8 spaces
puts(
  <<~FOO
    this line is indented 9 spaces
    this line is indented 1 space and 1 tab
  FOO
    .inspect
)

# 1 tab is greater than 7 spaces, the tab should be considered content
puts(
  <<~FOO
    this line is indented 7 spaces
    	this line is indented 1 tab
  FOO
    .inspect
)

# trailing spaces should not be removed
puts(
  <<-FOO
    this line and the next have a trailing space: 
     
    this line and the next have a trailing tab:	
	
  FOO
    .inspect
)

# trailing spaces should not be removed (squiggly)
puts(
  <<~FOO
    this line and the next have a trailing space: 
     
    this line and the next have a trailing tab:	
    	
  FOO
    .inspect
)

# single line of trailing space
puts(
  <<-FOO
   
  FOO
    .inspect
)

# single line of trailing space (squiggly)
puts(
  <<~FOO
   
  FOO
    .inspect
)

# multiple lines of trailing space
puts(
  <<-FOO
 
 
  FOO
    .inspect
)

# multiple lines of trailing space (squiggly)
puts(
  <<~FOO
 
 
  FOO
    .inspect
)
