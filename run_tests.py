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
from tests.test_integration import test_integrated_operation
from icn.blockchain.chain import Blockchain

def run_tests():
    # Create a test suite
    test_suite = unittest.TestSuite()

    # Add test cases
    loader = unittest.TestLoader()
    test_suite.addTest(loader.loadTestsFromTestCase(TestBlockchain))
    test_suite.addTest(loader.loadTestsFromTestCase(TestCooperative))
    test_suite.addTest(loader.loadTestsFromTestCase(TestFederation))
    test_suite.addTest(loader.loadTestsFromTestCase(TestConsensus))
    test_suite.addTest(loader.loadTestsFromTestCase(TestDIDManager))
    test_suite.addTest(loader.loadTestsFromTestCase(TestSmartContract))

    # Run the tests
    runner = unittest.TextTestRunner(verbosity=2)
    runner.run(test_suite)

    # Run integration test
    print("\nRunning Integration Test:")
    blockchain = Blockchain()
    blockchain.initialize()
    test_integrated_operation(blockchain)

if __name__ == '__main__':
    run_tests()