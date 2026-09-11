class Thing
    attr_reader :items

    def go(count)
        items
        build 1
        self.class
        self.then { |x| x }
        self.Integer
        self.count
        self.flag = true
        self[0]
        self + 1
        each { |x| x }
        items
        Integer()
        build.upcase
        build { 1 }
        count()
        [1].each { it() }
        build 1
        x.build 1
        x = build 1
        @y ||= build 1
        return build 1 if x
        return build(1), 2
        build(1).upcase
        build(1) + 1
        !build(1)
        [build(1)]
        p build(1)
        x ? build(1) : 2
        "#{build(1)}"
        (build(1))
        build(-1)
        build(*items)
        build(&blk)
        build 1, &blk
        build([1])
        build((1))
        build(/re/)
        build(%w[a])
        build(::A)
        build({ a: 1 })
        build a: 1
        build(1) { |x| x }
        build(x = 1)
        build(a && b)
        build(not a)
        build(a ? b : c)
        build(principal, capabilities:)
        build(
            1, # a comment inside the parentheses keeps them
            2,
        )
        build(
            a_very_long_first_argument_name,
            a_very_long_second_argument_name,
            a_very_long_third_argument_name,
        )
        build(
            a_very_long_first_argument_name,
            a_very_long_second_argument_name,
        ) { |x| x }
        build(checked_out: Array.new(count) { x }, deleted: [])
        super 1
        super()
        super
        yield 1
        x = yield 1
        [yield(1)]
        build bar(1)
        x if build(1)
        case status = build(1)
        when 1
            x
        end
        expect(1)
        count = build 1
        count(1)
    end

    def build(*)
    end

    def endless = build(1)
end
