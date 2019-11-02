Rails.application.routes.draw do
  resources :instances
  resources :playlists
  get '/songs/resolve' => 'songs#new_resolve', as: 'resolve_song'
  post '/songs/resolve' => 'songs#resolve'
  post '/callback/resolve' => 'callback#resolve'
  post '/callback/instance' => 'callback#instance'
  resources :songs
  devise_for :users
  # For details on the DSL available within this file, see https://guides.rubyonrails.org/routing.html
  root to: "user#index"
end
