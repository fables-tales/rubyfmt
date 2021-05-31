class A
  def self.m
    operation_name
    # @@formatted_op_names.compute_if_absent(operation_name) do
    #   operation_name.to_s.split('_').collect(&:capitalize).join
    # end
  end
end
