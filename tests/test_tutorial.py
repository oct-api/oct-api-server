#!/usr/bin/env python3
import unittest
import base

class TestTutorial(base.BaseTest):
    def do_chap01(self):
        self.create_app("testapp")
        app = self.get_app()[0]
        self.assertEqual(app['info']['name'], "testapp")
        self.assertEqual(app['status'], "PENDING")

        r = self.get(app['base_uri'] + '/__oct_status')
        self.assertEqual(r['status'], "PENDING")

        self.upgrade_app(app, "ch01", "ch01.yml")

        r = self.get(app['base_uri'] + '/__oct_status')
        self.assertEqual(r['status'], "RUNNING")

        r = self.get(app['base_uri'] + '/todo')
        self.assertEqual(r, [])

        data = {
            'subject': 'item 1',
        }
        self.post(app['base_uri'] + '/todo', data)

        r = self.get(app['base_uri'] + '/todo')
        self.assertEqual(len(r), 1)

        data = {
            'id': r[0]['id']
        }
        r = self.delete(app['base_uri'] + '/todo', data)

        r = self.get(app['base_uri'] + '/todo')
        self.assertEqual(r, [])

    def do_chap02(self):
        app = self.get_app()[0]
        self.upgrade_app(app, "ch02", "ch02.yml")

        data = {
            'subject': 'item 2',
        }
        self.post(app['base_uri'] + '/todo', data)

        r = self.get(app['base_uri'] + '/todo')
        self.assertEqual(len(r), 1)
        self.assertFalse(r[0]['done'])

        data = {
            'name': 'list 0',
        }
        self.post(app['base_uri'] + '/list', data)

        r = self.get(app['base_uri'] + '/list')
        self.assertEqual(len(r), 1)

        data = {
            'subject': 'item 2 on list 0',
            'list': r[0]['id'],
        }
        self.post(app['base_uri'] + '/todo', data)

        r = self.get(app['base_uri'] + '/todo')

    def do_chap03(self):
        app = self.get_app()[0]
        admin_token = app['info']['admin_token']
        self.upgrade_app(app, "ch03", "ch03.yml")

        r = self.do_get(app['base_uri'] + '/todo')
        self.assertTrue(r.status_code, 401)

        r = self.do_get(app['base_uri'] + '/list')
        self.assertTrue(r.status_code, 401)

        r = self.do_get(app['base_uri'] + '/foo')
        self.assertTrue(r.status_code, 404)

        r1 = self.get(app['base_uri'] + '/todo', token=admin_token)

        data = {
            'subject': 'item 3',
        }
        r = self.do_post(app['base_uri'] + '/todo', data)
        self.assertTrue(r.status_code, 401)

        r2 = self.get(app['base_uri'] + '/todo', token=admin_token)
        self.assertEqual(r2, r1)

        data = {
            'subject': 'item 3',
        }
        r = self.post(app['base_uri'] + '/todo', data, token=admin_token)

        r3 = self.get(app['base_uri'] + '/todo', token=admin_token)
        self.assertEqual(len(r3), len(r2) + 1)

    def do_user_reqs(self, app, username, nitems):
        admin_token = app['info']['admin_token']
        user_token = "%stoken" % username
        data = {
            "name": username,
            "email": "%s@example.com" % username,
            "pass": "dummypassword",
            "token": user_token,
        }
        r = self.post(app['base_uri'] + '/auth/user', data=data, token=admin_token)

        r = self.do_get(app['base_uri'] + '/auth/user', token=user_token)
        self.assertTrue(r.status_code, 401)

        for i in range(nitems):
            data = {
                'subject': '%s item %d' % (username, i),
            }
            self.post(app['base_uri'] + '/todo', data, token=user_token)

        r = self.get(app['base_uri'] + '/todo', token=user_token)
        self.assertEqual(len(r), nitems)

    def do_chap04(self):
        app = self.get_app()[0]
        admin_token = app['info']['admin_token']
        self.upgrade_app(app, "ch04", "ch04.yml")

        r = self.do_get(app['base_uri'] + '/auth/user')
        self.assertTrue(r.status_code, 401)

        r = self.get(app['base_uri'] + '/auth/user', token=admin_token)
        self.assertEqual(r, [])

        r1 = self.get(app['base_uri'] + '/todo', token=admin_token)

        self.do_user_reqs(app, "user1", 3)
        self.do_user_reqs(app, "user2", 4)
        self.do_user_reqs(app, "user3", 5)

        r = self.get(app['base_uri'] + '/auth/user', token=admin_token)
        self.assertEqual(len(r), 3)

        r2 = self.get(app['base_uri'] + '/todo', token=admin_token)
        self.assertEqual(len(r2), len(r1) + 3 + 4 + 5)

    def test_tutorial(self):
        self.do_chap01()
        self.do_chap02()
        self.do_chap03()
        self.do_chap04()

if __name__ == '__main__':
    unittest.main()
