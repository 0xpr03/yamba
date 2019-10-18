class CreateEntries < ActiveRecord::Migration[6.0]
  def change
    create_table :entries do |t|
      t.belongs_to :playlist, null: false, foreign_key: true
      t.belongs_to :song, null: false, foreign_key: true

      t.timestamps
    end
  end
end
