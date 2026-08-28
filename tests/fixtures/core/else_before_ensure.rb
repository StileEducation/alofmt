begin
    a
rescue StandardError
    b
else
    c
# d
ensure
    e
end
def foo
    a
rescue StandardError
    b
else
    c
# d
# e
ensure
    e
end
foo do
    a
rescue StandardError
    b
else
    c
# d
ensure
    e
end
