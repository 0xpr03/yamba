require "application_system_test_case"

class InstancesTest < ApplicationSystemTestCase
  setup do
    @instance = instances(:one)
  end

  test "visiting the index" do
    visit instances_url
    assert_selector "h1", text: "Instances"
  end

  test "creating a Instance" do
    visit instances_url
    click_on "New Instance"

    fill_in "Api token", with: @instance.api_token
    check "Autostart" if @instance.autostart
    fill_in "Cid", with: @instance.cid
    fill_in "Host", with: @instance.host
    fill_in "Identity", with: @instance.identity
    fill_in "Name", with: @instance.name
    fill_in "Password", with: @instance.password
    fill_in "Port", with: @instance.port
    click_on "Create Instance"

    assert_text "Instance was successfully created"
    click_on "Back"
  end

  test "updating a Instance" do
    visit instances_url
    click_on "Edit", match: :first

    fill_in "Api token", with: @instance.api_token
    check "Autostart" if @instance.autostart
    fill_in "Cid", with: @instance.cid
    fill_in "Host", with: @instance.host
    fill_in "Identity", with: @instance.identity
    fill_in "Name", with: @instance.name
    fill_in "Password", with: @instance.password
    fill_in "Port", with: @instance.port
    click_on "Update Instance"

    assert_text "Instance was successfully updated"
    click_on "Back"
  end

  test "destroying a Instance" do
    visit instances_url
    page.accept_confirm do
      click_on "Destroy", match: :first
    end

    assert_text "Instance was successfully destroyed"
  end
end
