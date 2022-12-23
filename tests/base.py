#!/usr/bin/env python3
import os
import time
import subprocess
import tempfile
import shutil
import unittest
import json
import requests

class BaseTest(unittest.TestCase):
    def setUp(self):
        self.base_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        orm_port = 19892
        orm_addr = "127.0.0.1:%d" % orm_port
        self.orm_api = "http://" + orm_addr
        server_port = 19899
        server_addr = "127.0.0.1:%d" % server_port
        self.oct_api = "http://" + server_addr
        data_dir = tempfile.mkdtemp()
        self.tmpdir = data_dir

        # Make sure we don't need to wait for build when we do cargo run
        subprocess.check_output(['cargo', 'build'], cwd=self.base_dir)

        env = dict(os.environ.items())
        env['OCT_DATA_DIR'] = data_dir

        cmd = ['./orm/manage.py', 'migrate']
        subprocess.check_output(cmd, cwd=self.base_dir, env=env)

        cmd = ['./orm/manage.py', 'runserver', orm_addr]
        self.orm = subprocess.Popen(cmd, cwd=self.base_dir, env=env)
        self.wait_server(self.orm_api)

        cmd = ['cargo', 'run', '--',
               '--no-start-orm',
               '--orm-addr', orm_addr,
               '--data', data_dir,
               '--addr', server_addr]
        self.server = subprocess.Popen(cmd, cwd=self.base_dir, env=env)
        self.wait_server(self.oct_api + "/status")

        self.assertIsNone(self.orm.poll())
        self.assertIsNone(self.server.poll())
        self.create_user()

    def tearDown(self):
        self.orm.terminate()
        self.orm.wait()
        self.server.terminate()
        self.server.wait()
        shutil.rmtree(self.tmpdir)

    def wait_server(self, url):
        for i in range(20):
            try:
                r = requests.get(url)
            except Exception as e:
                time.sleep(0.5)
                continue
            if r.ok:
                return
            time.sleep(0.5)
        raise Exception("Cannot connect to server")

    def trace_response(self, r):
        if not r.ok:
            print(r, r.text)

    def orm_get(self, path):
        r = requests.get(self.orm_api + path)
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def orm_post(self, path, data):
        r = requests.post(self.orm_api + "/user/", json=data)
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def create_user(self):
        self.username = "testuser"
        self.token = "testtoken"
        r = requests.post(self.orm_api + "/user/", json={
            'username': self.username,
            'email': "test@oct-api.com",
            'display_name': "Test User",
            'token': self.token,
            })
        self.assertTrue(r.ok)

    def headers(self, token=None):
        return {
            'Authorization': 'token ' + (token or self.token),
            'Content-type': 'application/json',
        }

    def do_get(self, path, token=None):
        return requests.get(self.oct_api + path, headers=self.headers(token))

    def get(self, path, token=None):
        r = self.do_get(path, token)
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def do_post(self, path, data, token=None):
        return requests.post(self.oct_api + path, data=json.dumps(data),
                          headers=self.headers(token))

    def post(self, path, data, token=None):
        r = self.do_post(path, data, token)
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def put(self, path, data):
        r = requests.put(self.oct_api + path, data=json.dumps(data),
                          headers=self.headers())
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def delete(self, path, data):
        r = requests.delete(self.oct_api + path, data=json.dumps(data),
                          headers=self.headers())
        self.trace_response(r)
        self.assertTrue(r.ok)
        return r.json()

    def create_app(self, name):
        data = {
            'name': name,
        }
        self.post("/meta/app", data)

    def delete_app(self, name):
        data = {
            'name': name,
        }
        self.delete("/meta/app", data)

    def get_app(self):
        return self.get("/meta/app")

    def create_app_repo(self, name, ymlname):
        ymlpath = os.path.join(self.base_dir, "tests/data/", ymlname)
        repo = os.path.join(self.tmpdir, "repos/", name)
        subprocess.check_output("""
        set -e
        mkdir -p {repo}
        cd {repo}
        git init
        cp {yml} app.yml
        git config user.email "test@oct-api.com"
        git config user.name "Oct API"
        git add app.yml
        git commit -m 'initial commit'
        """.format(repo=repo, yml=ymlpath), shell=True, cwd=self.tmpdir)
        return repo

    def upgrade_app(self, app, name, ymlname):
        repo = self.create_app_repo(name, ymlname)
        data = app['info']
        data['git_repo'] = repo
        self.put("/meta/app", data)
        self.post("/meta/sync", {
            'name': data['name']
        })
