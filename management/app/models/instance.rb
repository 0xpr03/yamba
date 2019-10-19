class Instance < ApplicationRecord
  has_secure_token :api_token

  validates :host, presence: true
  validates :name, presence: true
  after_create :daemon_start
  after_update :daemon_update
  after_destroy :daemon_stop

  def daemon_start
    HTTP.headers(:content_type => "application/json")
             .post("http://172.18.0.2:1338/instance/start", :json => {
                 :id => self.id,
                 :data => {
                     :TS => self
                 },
                 :auth_token => self.api_token,
                 :volume => 0.05
             })
  end

  def daemon_update
    daemon_stop
    daemon_start
  end

  def daemon_stop
    HTTP.headers(:content_type => "application/json")
        .post("http://172.18.0.2:1338/instance/stop", :json => {
            :id => self.id
        })
  end
end
