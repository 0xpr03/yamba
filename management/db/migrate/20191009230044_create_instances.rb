class CreateInstances < ActiveRecord::Migration[6.0]
  def change
    create_table :instances do |t|
      t.string :host, null: false
      t.integer :port, limit: 16
      t.string :identity
      t.integer :cid, limit: 32
      t.string :name, null: false
      t.string :password
      t.boolean :autostart
      t.string :api_token

      t.timestamps
    end

    add_index :instances, :api_token, unique: true
  end
end
