class Instance < ApplicationRecord
  has_secure_token :api_token

  validates :host, presence: true
  validates :name, presence: true
  after_create :start_instance
  after_update :update_instance
  after_destroy :stop_instance

  def start_instance
    HTTP.headers(:content_type => "application/json")
             .post("http://172.18.0.3:1338/instance/start", :json => {
                 :id => self.id,
                 :data => {
                     :TS => self
                 },
                 :volume => 0.05
             })
  end

  def update_instance
    stop_instance
    start_instance
  end

  def stop_instance
    HTTP.headers(:content_type => "application/json")
        .post("http://172.18.0.3:1338/instance/stop", :json => {
            :id => self.id
        })
  end
end
