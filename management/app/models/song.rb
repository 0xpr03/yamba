class Song < ApplicationRecord
  has_many :entries
  has_many :playlists, through: :entries
end
