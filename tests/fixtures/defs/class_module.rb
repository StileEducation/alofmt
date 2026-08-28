class A
end
class A
end
class A < B
    x
end
class A::B < C::D
end
class ::A
end
class A < b.c
end
class A < ::B::C
    x
end
class A < Struct.new(:a, :b)
    x
end
class A < Struct.new(
    :aaaaaaaaaaaaaaaaaaaaaaa,
    :bbbbbbbbbbbbbbbbbbbbbbbbbbb,
    :cccccccccccccccccccccccc,
)
end
class Aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa < Bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
end
class << self
    x
end
class << self

end
class << foo.bar

end
class << self
    # only comment
end
class << self # trailing

end
class << self
    def foo
        1
    end
end
module A
end
module A::B
    x
end
class A
    # comment
end
class A
    x
    # comment
end
class A < B # trailing
    x
end
module M # trailing
end
# leading
module Atlas
    module Services
        # A class comment
        class Foo < ::Bar::Baz
            extend T::Sig
            include Comparable

            attr_reader :a, :b

            # Comment on initialize
            sig { params(a: Integer, b: T.nilable(String)).void }
            def initialize(a, b: nil)
                puts a
            end

            def self.build(opts = {}, &block)
                new(opts).tap(&block)
            end

            def <=>(other)
                a <=> other.a
            end
            # A comment between methods

            private

            def empty
            end

            protected

            # trailing comment before end
        end

        class << self
            def singleton_method
                1
            end
        end
    end
end
Struct.new(:a, :b) do
    def c
        a + b
    end
end
