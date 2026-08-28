def x
    metaclass.send(
        :define_method,
        rpc_method,
    ) { |actor, req, options = [], x = 1| correlation_id = 1 }
    logger_class.send(:define_method, level) do |msg = nil, data = {}, &block|
        correlation_id = 1
    end
end
