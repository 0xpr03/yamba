json.extract! song, :id, :name, :source, :artist, :length, :created_at, :updated_at
json.url song_url(song, format: :json)
