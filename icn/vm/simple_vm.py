# icn/blockchain/simple_vm.py

class SimpleVM:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.stack = []
        self.variables = {}

    def execute(self, contract):
        pc = 0
        while pc < len(contract.parsed_code):
            op = contract.parsed_code[pc]
            if op[0] == 'transfer':
                self.execute_transfer(op[1], op[2], op[3])
            elif op[0] == 'vote':
                self.execute_vote(op[1], op[2])
            elif op[0] == 'if':
                if not self.execute_condition(op[1], op[2], op[3]):
                    pc = self.find_matching_endif(contract.parsed_code, pc)
            elif op[0] == 'else':
                pc = self.find_matching_endif(contract.parsed_code, pc)
            pc += 1

    def execute_transfer(self, from_did, to_did, amount):
        if self.blockchain.get_balance(from_did) >= amount:
            self.blockchain.add_transaction(self.blockchain.create_transaction(from_did, to_did, amount))
            return True
        return False

    def execute_vote(self, voter_did, proposal_id):
        for dao in self.blockchain.dao_manager.daos.values():
            if proposal_id in dao.proposals and voter_did in dao.members:
                return dao.vote_on_proposal(proposal_id, voter_did, True)
        return False

    def execute_condition(self, var1, op, var2):
        val1 = self.variables.get(var1, 0)
        val2 = self.variables.get(var2, 0)
        if op == '==':
            return val1 == val2
        elif op == '>':
            return val1 > val2
        elif op == '<':
            return val1 < val2
        return False

    def find_matching_endif(self, code, start):
        count = 1
        for i in range(start + 1, len(code)):
            if code[i][0] == 'if':
                count += 1
            elif code[i][0] == 'endif':
                count -= 1
                if count == 0:
                    return i
        return len(code) - 1