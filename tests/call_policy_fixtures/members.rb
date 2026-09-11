require 'json'
puts 'a top-level statement is a macro and keeps its command form'

class Job < Base
    include Comparable
    attr_reader :queue, :name
    const :status, String
    alias title name

    def run
        self.queue().push(self.build())
        self.queue()
            .select { |item| item.ready? }
            .map { |item| item.name }
            .reject { |name| name.empty? }
            .first
        puts(self.status())
        puts(self.name()) if self.verbose()
        self.log('start', level: :info)
        self.helper(1, 2) { |x| x }
        self.each_item { |item| item }
        items.map { |i| i }
        unless self.ok?()
            raise ArgumentError, 'allowed methods keep their command form'
        end
        yield 1
        super(1)
        self.name().upcase
        self.title()
        format
        verbose?
        self.class.count
        name = 1
        name
        self.ok?() ? self.build() : self.queue()
        self.build(a ? b : c)
        self.apply(-1)
        self.apply(*items)
        self.apply(&block)
        items[0] = self.build()
        self.flag = true
        owner + items
        x = [self.build(), owner]
        Integer(items)
        owner&.notify(items)
    end

    def build(*)
    end

    def each_item
    end

    def ok?
    end

    def log(message, level:)
    end

    def helper(a, b)
    end

    def verbose
    end

    def apply(*)
    end

    def self.count
        self.total()
        build
    end

    def self.total
    end

    class << self
        def registry
            self.total()
            self.registry()
        end
    end

    private def secret
        self.build()
    end

    define_method(:dynamic) { self.build() }
end

module Helpers
    def helper
        self.other()
    end

    def other
    end
end

describe Job do
    let(:job) { Job.new }

    it 'runs' do
        expect(job.run).to eq(1)
        make_thing 'a statement of a macro block is a macro'
    end
end
