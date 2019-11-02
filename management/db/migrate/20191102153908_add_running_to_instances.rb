class AddRunningToInstances < ActiveRecord::Migration[6.0]
  def change
    add_column :instances, :running, :boolean, :default => false
  end
end
