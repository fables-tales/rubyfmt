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
puts \
<<~FOO.inspect
  the next line contains a single space
 
FOO

# 1 tab equals 8 spaces, both should be considered indentation
puts \
<<~FOO.inspect
        this line is indented 8 spaces
	this line is indented 1 tab
FOO

# 1 tab plus 1 space equals 8 spaces
puts \
<<~FOO.inspect
         this line is indented 9 spaces
	 this line is indented 1 space and 1 tab
FOO

# 1 tab is greater than 7 spaces, the tab should be considered content
puts \
<<~FOO.inspect
       this line is indented 7 spaces
	this line is indented 1 tab
FOO

# trailing spaces should not be removed
puts \
<<-FOO.inspect
    this line and the next have a trailing space: 
     
    this line and the next have a trailing tab:	
	
FOO

# trailing spaces should not be removed (squiggly)
puts \
<<~FOO.inspect
    this line and the next have a trailing space: 
     
    this line and the next have a trailing tab:	
	
FOO

# single line of trailing space
puts \
<<-FOO.inspect
   
FOO

# single line of trailing space (squiggly)
puts \
<<~FOO.inspect
   
FOO

# multiple lines of trailing space
puts \
<<-FOO.inspect
 
 
FOO

# multiple lines of trailing space (squiggly)
puts \
<<~FOO.inspect
 
 
FOO
