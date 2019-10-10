Rails.application.routes.draw do
  resources :instances
  resources :playlists
  resources :songs
  devise_for :users
  # For details on the DSL available within this file, see https://guides.rubyonrails.org/routing.html
  root to: "user#index"
end
