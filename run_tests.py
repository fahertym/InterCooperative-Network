# run_tests.py

import unittest

# Import your test modules here
from tests.test_blockchain import TestBlockchain
from tests.test_cooperative import TestCooperative
from tests.test_federation import TestFederation

def run_tests():
    # Create a test suite
    test_suite = unittest.TestSuite()

    # Add test cases
    test_suite.addTest(unittest.makeSuite(TestBlockchain))
    test_suite.addTest(unittest.makeSuite(TestCooperative))
    test_suite.addTest(unittest.makeSuite(TestFederation))

    # Run the tests
    runner = unittest.TextTestRunner(verbosity=2)
    runner.run(test_suite)

if __name__ == '__main__':
    run_tests()