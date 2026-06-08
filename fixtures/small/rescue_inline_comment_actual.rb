begin
rescue Foo # comment
end

begin
rescue Foo,
       Bar # comment
end

begin
rescue \
    Foo # comment
end

begin
rescue \
    Foo,
    Bar # comment
end
