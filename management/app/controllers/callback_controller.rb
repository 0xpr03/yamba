class CallbackController < ApplicationController
  skip_before_action :verify_authenticity_token, only: [:resolve]

  def resolve
    puts params[:callback]
  end

  private
    # Never trust parameters from the scary internet, only allow the white list through.
    def callback_params
      params.require(:song).permit(:name, :source, :artist, :length)
    end
end