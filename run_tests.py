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
    loader = unittest.TestLoader()
    test_suite = unittest.TestSuite()

    test_suite.addTest(loader.loadTestsFromTestCase(TestBlockchain))
    test_suite.addTest(loader.loadTestsFromTestCase(TestCooperative))
    test_suite.addTest(loader.loadTestsFromTestCase(TestFederation))
    test_suite.addTest(loader.loadTestsFromTestCase(TestConsensus))
    test_suite.addTest(loader.loadTestsFromTestCase(TestDIDManager))
    test_suite.addTest(loader.loadTestsFromTestCase(TestSmartContract))

    runner = unittest.TextTestRunner(verbosity=2)
    runner.run(test_suite)

if __name__ == '__main__':
    run_tests()