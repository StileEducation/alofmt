x =
    begin
        a ? b : c
    rescue StandardError
        d
    end
begin
    b if a
rescue StandardError
    c
end
foo(
    (
        begin
            a ? b : c
        rescue StandardError
            d
        end
    ),
)
