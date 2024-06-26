# run_tests.py

import unittest
from tests.test_all import (
    TestBlockchain,
    TestCooperative,
    TestFederation,
    TestConsensus,
    TestDIDManager,
    TestSmartContract
)

def run_tests():
    # Create a test suite
    test_suite = unittest.TestSuite()

    # Add test cases
    test_suite.addTest(unittest.makeSuite(TestBlockchain))
    test_suite.addTest(unittest.makeSuite(TestCooperative))
    test_suite.addTest(unittest.makeSuite(TestFederation))
    test_suite.addTest(unittest.makeSuite(TestConsensus))
    test_suite.addTest(unittest.makeSuite(TestDIDManager))
    test_suite.addTest(unittest.makeSuite(TestSmartContract))

    # Run the tests
    runner = unittest.TextTestRunner(verbosity=2)
    runner.run(test_suite)

if __name__ == '__main__':
    run_tests()