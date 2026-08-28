foo do
    b
rescue StandardError
    c
end
foo do
    b
ensure
    c
end
foo do
    b
rescue A
    c
else
    d
ensure
    e
end
foo do

rescue StandardError
end
foo do
    # c
rescue StandardError
    # c2
end
foo bar do
    b
rescue StandardError
    c
end
-> do
    b
rescue StandardError
    c
end
