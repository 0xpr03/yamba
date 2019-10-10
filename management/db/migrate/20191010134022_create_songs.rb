class CreateSongs < ActiveRecord::Migration[6.0]
  def change
    create_table :songs do |t|
      t.string :name
      t.string :source
      t.string :artist
      t.integer :length

      t.timestamps
    end
  end
end
