# Copyright 2021-2023 FLECS Technologies GmbH
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

import requests
import json
import time
import os

mp_url = "https://marketplace.flecs.tech:8443"
local_mp_url = "http://localhost:8001"
flecs_core_url = "http://localhost:8951"
flecs_webapp_url = "http://localhost:80"

# Login information
mp_user = os.environ["MP_USER"]
mp_password = os.environ["MP_PASSWORD"]
login_response = None

# Test apps
system_apps = [
    "tech.flecs.service-mesh",
    "tech.flecs.mqtt-bridge"
]
user_apps = [
    "org.mosquitto.broker",
    "io.anyviz.cloudadapter",
    "com.grafana.grafana-oss",
    "com.influxdata.influxdb"]
user_apps_versions = [
    "2.0.14-openssl",
    "0.9.6.0",
    "9.3.1",
    "2.5.1"]
user_apps_ports= {
    "io.anyviz.cloudadapter": 8888,
    "com.grafana.grafana-oss": 3000,
    "com.influxdata.influxdb": 8086
}

###
### Utility
###

def format_json_data(json_input: dict) -> str:
    """
    Format json input as plain string, but remove whitespaces after , and :
    """

    # usage: requests.post(url, data=format_json_data(payload))
    # requests.post can also take json=payload as argument.
    # However, that adds extra whitespaces after : and ,
    # e.g. {"username": "development-customer@flecs.tech", "password"...
    # which the server does not like (returns 500)
    return json.dumps(json_input, separators=(',', ':'))

def authenticate():
    """
    Authenticate to marketplace and receive JWT.
    """

    auth_url = mp_url + "/api/access/authenticate"
    header = {"content-type":"application/json"}
    payload = {"username":mp_user, "password":mp_password, "issueJWT":True}
    resp = requests.post(auth_url,
        headers=header,
        data=format_json_data(payload))
    login_response = resp.json(),
    return login_response[0]

def get_tickets() -> list:
    """"
    Get unused tickets
    """

    license_url = mp_url + "/api/license/get-current-user-licenses"
    files = {
        "aam-jwt": (None, login_response["jwt"]["token"])
    }
    resp = requests.post(license_url, files=files)
    all_tickets= resp.json()["response"]["licenses"]
    is_license_unused = list(t["activation_date"] == None for t in all_tickets) # contains True if license is unused, False if license is used
    unused_tickets = [all_tickets[i] for i in range(len(is_license_unused)) if is_license_unused[i]==True]
    return unused_tickets

def count_tickets()-> int:
    """
    Count available tickets.
    """

    unused_tickets = get_tickets()
    ticket_count = len (unused_tickets)
    return ticket_count

def get_license(offset=0)-> str:
    """"
    Get a license
    """

    tickets = get_tickets()
    license = tickets[offset]["license_key"]
    return license

def get_jobs():
    """
    Get list of currently running jobs.
    """

    jobs_url = flecs_core_url + "/v2/jobs"

    resp = requests.get(jobs_url)
    return resp.json()

def get_ongoing_job_ids():
    """
    Get the IDs of running or queued jobs.
    """

    jobs = get_jobs()
    ids = [i for i in range(len(jobs)) if jobs[i]["status"] not in ("cancelled", "successful", "failed")]
    return ids

def wait_til_job_finished(jobId: int) -> dict:
    """
    Wait until the job with the given ID has finished and return its result
    """

    ongoing_job_sleep_time = 1

    while True:
        job_url = flecs_core_url + "/v2/jobs/" + str(jobId)
        resp = requests.get(job_url)
        res = resp.json()[0]
        if res["status"] not in ("cancelled", "successful", "failed"): # job is still running
            time.sleep(ongoing_job_sleep_time)
            continue
        else:
            break

    return res["result"]

###
### Test: Check version
###

def version() -> str:
    """
    Check FLECS core version.
    """

    version_url = flecs_core_url + "/v2/system/version"
    resp = requests.get(version_url)
    version = resp.json()['core']
    return version

def test_version():
    """
    TC01: Check version of FLECS core.
        Go to the system page and check if the desired version is installed.
    """

    version_url = mp_url + "/dl/latest_flecs_amd64"

    resp = requests.get(version_url)
    expected_version = resp.text

    actual_version = version()
    assert expected_version == actual_version
    # TODO actual version contains dash

###
### Test: Rate app
###

def rate_app(app_id: int, app_rating:int) -> dict:
    """
    Give app with id app_id a rating of app_rating.
    """

    rate_url = mp_url + "/api/v1/products/reviews"

    payload = {
        "data":{
            "product_id": app_id,
            "review": "This is an in-app rating without review.",
            "reviewer": login_response['user']['data']['user_login'],
            "reviewer_email": mp_user,
            "rating": app_rating
        },
        "jwt": login_response['jwt']['token']
    }

    resp = requests.post(rate_url, data=format_json_data(payload))
    # TODO gives 500 when re-submitting a rating. Find a way to remove rating afterwards
    return resp

def test_rate_app():
    """
    TC04: Rate an app in the FLECS marketplace.
        Go to the marketplace page and rate an app.
        Switch to mp-dev admin backend and check if the rating is available for approval.
    """

    resp = rate_app(751, 5)
    assert True == resp.ok

###
### Test: Instances
###

def get_instances() -> list:
    """
    Get instances from FLECS core
    """

    instance_url = flecs_core_url + "/v2/instances"
    resp = requests.get(instance_url)
    instance_list = resp.json()
    return instance_list

def create_instance(appname: str, appversion: str) -> str:
    """
    Create a new instance of the given app name and version
    """

    instance_url = flecs_core_url + "/v2/instances/create"

    header = {"content-type":"application/json"}

    payload = {
        "appKey": {
            "name": appname,
            "version": appversion
        },
        "instanceName": appname + "0"
    }

    resp = requests.post(instance_url, headers=header, data=format_json_data(payload))
    return resp

def start_instance(instanceId:str) -> str:
    """
    Start the instance given by instanceId
    """

    start_url = flecs_core_url + "/v2/instances/" + instanceId + "/start"
    resp = requests.post(start_url)

    return resp

###
### Test: Installed apps are correct
###

def get_apps() -> list:
    """
    Get installed apps from FLECS core
    """

    app_list_url = flecs_core_url + "/v2/apps"
    resp = requests.get(app_list_url)
    app_list = resp.json()
    return app_list

def test_get_apps():
    """
    TC02: Read app list of freshly installed FLECS.
    Expect:
      - length 2
      - app "tech.flecs.service-mesh" is "INSTALLED"
      - single instance of app "tech.flecs.service-mesh" is "RUNNING"
      - app "tech.flecs.mqtt-bridge" is "INSTALLED"
      - single instance of app "tech.flecs.mqtt-bridge" is "RUNNING"
    """

    # get list of installed apps
    app_list = get_apps()

    # get instances
    instance_list = get_instances()

    # expected installed apps:
    expected_installed_apps = system_apps

    # assert correct length
    assert len(expected_installed_apps) == len(app_list)

    # expected apps are installed and a single instance is running
    for app_name in expected_installed_apps:
        # assert that app-name occurs in app_list exactly once
        app_matches_idx = list(idx for idx in range(len(app_list)) if app_list[idx]["appKey"]["name"] == app_name)
        assert 1 == len(app_matches_idx)

        # assert that app is installed
        app_index = app_matches_idx[0]
        assert app_list[app_index]["status"] == "installed"

        # assert that single instance is running
        app_instance_idx = list(idx for idx in range(len(instance_list)) if instance_list[idx]["appKey"]["name"] == app_name)
        assert 1 == len(app_instance_idx)
        assert "running" == instance_list[app_instance_idx[0]]["status"]

    # TODO: system apps not listed in v2 api response yet -> test fails

###
### Test: Login with JWT works
###

def login() -> dict:
    """
    Pass login data to FLECS daemon
    """

    login_url = flecs_core_url + "/v2/marketplace/login"

    header = {"content-type":"application/json"}
    payload = {
        "token": login_response["jwt"]["token"],
        "user": login_response["user"]["data"]["user_login"]
    }

    resp = requests.post(login_url, headers=header, data=format_json_data(payload))
    return resp

def test_login():
    """
    TC03: Pass login data to FLECS daemon.
    Expect:
        - "OK" response
    """
    resp = login()
    assert True == resp.ok

###
### Test: App installation works
###

def install_app(user_app: str, user_app_version: str, offset):
    """
    Install user_app_version of user_app.
    """

    install_url = flecs_core_url + "/v2/apps/install"

    # make sure we have tickets
    license_count = count_tickets()
    assert license_count > 0

    # get a ticket
    license_key = get_license(offset)

    header = {"content-type": "application/json"}
    payload = {
        "app": user_app,
        "version": user_app_version,
        "licenseKey": license_key
    }

    resp = requests.post(install_url, headers=header, data=format_json_data(payload))

def test_install_apps():
    """
    TC05: App installation
    """

    # count tickets beforehand
    ticked_count_pre = count_tickets()

    # make sure app and version lists have same size
    app_count = len(user_apps)
    assert app_count == len(user_apps_versions)

    # install apps
    for i in range(app_count):
        install_app(user_apps[i], user_apps_versions[i], i)

    # wait for jobs to finish
    ongoing_job_sleep_time = 1
    unfinished_job_ids = get_ongoing_job_ids()
    while unfinished_job_ids:
        time.sleep(ongoing_job_sleep_time)
        unfinished_job_ids = get_ongoing_job_ids()

    # check if apps are installed
    installed_apps = get_apps()
    for user_app in user_apps:
        assert user_app in list(installed_app["appKey"]["name"] for installed_app in installed_apps)
        # installed_apps is a list of dicts, we need to look at the "app" entry of each dict

    # check if correct amount of tickets were consumed
    ticked_count_post = count_tickets()
    tickets_consumed = ticked_count_pre - ticked_count_post
    assert len(user_apps) <= tickets_consumed # until there is a separate testing account, someone else might use tickets at the same time

###
### Test: Instances of installed apps can be created
###

def test_start_instances():
    """
    TC06: Start instances of newly installed apps
    """

    for i in range(len(user_apps)):
        appname = user_apps[i]
        appversion = user_apps_versions[i]
        resp = create_instance(appname, appversion)
        assert True == resp.ok
        create_jobId = resp.json()["jobId"]
        # wait_til_jobs_finished()
        res = wait_til_job_finished(create_jobId)
        if (res["code"] == 0):
            instanceId = res["message"]
            resp = start_instance(instanceId)
            assert True == resp.ok
            if resp.json():
                start_jobId = resp.json()["jobId"]
                wait_til_job_finished(start_jobId)

###
### Test: installed apps can be opened
###

def open_app(app: str):
    """"
    Send GET request to port corresponding to app
    """

    # TODO: ports need to be forwarded from inside docker image to host. Currently hardcoded in scripts/build-image.sh

    port = user_apps_ports[app]
    url = "http://localhost:" + str(port)
    resp = requests.get(url)

    return resp

def test_open_apps():
    """
    TC07: Test if opening the apps in user_apps_ports works
    """

    time.sleep(5) # wait for instances to become responsive

    for app in user_apps_ports:
        resp = open_app(app)
        assert True == resp.ok

    # TODO: currently getting 'connection reset by peer' error

###
### Test: app uninstallation works
###

def uninstall_app(user_app: str, user_app_version: str):
    """
    Uninstall user_app_version of user_app.
    """

    uninstall_url = flecs_core_url + "/v2/apps/" +  user_app + "?version=" + user_app_version
    resp = requests.delete(uninstall_url)

def test_uninstall_app():
    """
    TC12: Uninstall user_apps
    """

    # make sure app and version lists have same size
    app_count = len(user_apps)
    assert app_count == len(user_apps_versions)

    # get installed apps
    installed_apps_pre = get_apps()

    # uninstall apps
    for i in range(app_count):
        uninstall_app(user_apps[i], user_apps_versions[i])

    # wait for jobs to finish
    ongoing_job_sleep_time = 1
    unfinished_job_ids = get_ongoing_job_ids()
    while unfinished_job_ids:
        time.sleep(ongoing_job_sleep_time)
        unfinished_job_ids = get_ongoing_job_ids()

    # check if apps are really uninstalled
    installed_apps_post = get_apps()
    assert len(installed_apps_post) == len(installed_apps_pre) - app_count

    for app in installed_apps_pre:
        # make sure app is no longer installed
        assert app not in installed_apps_post

###
### main
###

login_response = authenticate()
print('done!')
