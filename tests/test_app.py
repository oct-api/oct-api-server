#!/usr/bin/env python3
import unittest
import base

class TestApp(base.BaseTest):

    def test_app_delete(self):
        self.create_app("testapp")
        self.assertEqual(len(self.get_app()), 1)
        self.delete_app("testapp")
        self.assertEqual(self.get_app(), [])

if __name__ == '__main__':
    unittest.main()
