class Playlist < ApplicationRecord
  has_many :entries
  has_many :songs, through: :entries
end
