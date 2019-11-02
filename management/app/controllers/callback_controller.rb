class CallbackController < ApplicationController
  skip_before_action :verify_authenticity_token, only: [:resolve, :instance]

  def resolve
    data = resolve_params[:data]
    unless data[:state] == "Finished"
      # Implement (possibly websocket?) error message for user
      return false
    end
    Song.create(data[:data][:Song])
  end

  def instance
    instance = Instance.find(instance_params[:id])
    case instance_params[:state]
    when "Started"
      instance.running=true
    when "Running"
      instance.running=true
    when "Stopped"
      instance.running=false
    else
      instance.running=false
    end
    instance.save
  end

  private

    def resolve_params
      params.require(:callback).permit(:ticket, data: [ :state, data: [ Song: [ :name, :artist, :source, :length]]])
    end

    def instance_params
      params.require(:callback).permit(:state, :id)
    end
end