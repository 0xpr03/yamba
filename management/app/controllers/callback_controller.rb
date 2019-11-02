class CallbackController < ApplicationController
  skip_before_action :verify_authenticity_token, only: [:resolve]

  def resolve
    data = resolve_params[:data]
    unless data[:state] == "Finished"
      # Implement (possibly websocket?) error message for user
      return false
    end
    Song.create(data[:data][:Song])
  end

  private

    def resolve_params
      params.require(:callback).permit(:ticket, data: [ :state, data: [ Song: [ :name, :artist, :source, :length]]])
    end
end