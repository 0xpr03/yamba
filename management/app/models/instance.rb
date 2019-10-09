class Instance < ApplicationRecord
  has_secure_token :api_token
end
