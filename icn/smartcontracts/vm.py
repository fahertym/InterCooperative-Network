# icn/smartcontracts/vm.py

from .language import OpCode

class SmartContractVM:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.stack = []
        self.memory = {}
        self.pc = 0

    def execute(self, instructions):
        self.pc = 0
        while self.pc < len(instructions):
            instruction = instructions[self.pc]
            self.execute_instruction(instruction)
            self.pc += 1

    def execute_instruction(self, instruction):
        if instruction.opcode == OpCode.PUSH:
            self.stack.append(int(instruction.args[0]))
        elif instruction.opcode == OpCode.POP:
            self.stack.pop()
        elif instruction.opcode == OpCode.ADD:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a + b)
        elif instruction.opcode == OpCode.SUB:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a - b)
        elif instruction.opcode == OpCode.MUL:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a * b)
        elif instruction.opcode == OpCode.DIV:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(a // b)
        elif instruction.opcode == OpCode.STORE:
            value = self.stack.pop()
            address = self.stack.pop()
            self.memory[address] = value
        elif instruction.opcode == OpCode.LOAD:
            address = self.stack.pop()
            self.stack.append(self.memory.get(address, 0))
        elif instruction.opcode == OpCode.EQ:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(int(a == b))
        elif instruction.opcode == OpCode.LT:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(int(a < b))
        elif instruction.opcode == OpCode.GT:
            b, a = self.stack.pop(), self.stack.pop()
            self.stack.append(int(a > b))
        elif instruction.opcode == OpCode.JMP:
            self.pc = int(instruction.args[0]) - 1
        elif instruction.opcode == OpCode.JMPIF:
            condition = self.stack.pop()
            if condition:
                self.pc = int(instruction.args[0]) - 1
        elif instruction.opcode == OpCode.SET_PRICE:
            item = self.stack.pop()
            price = self.stack.pop()
            self.blockchain.set_price(item, price)
        elif instruction.opcode == OpCode.GET_PRICE:
            item = self.stack.pop()
            price = self.blockchain.get_price(item)
            self.stack.append(price)
        elif instruction.opcode == OpCode.TRADE:
            buyer = self.stack.pop()
            seller = self.stack.pop()
            item = self.stack.pop()
            quantity = self.stack.pop()
            self.blockchain.trade(buyer, seller, item, quantity)
        elif instruction.opcode == OpCode.GET_BALANCE:
            account = self.stack.pop()
            balance = self.blockchain.get_balance(account)
            self.stack.append(balance)
        elif instruction.opcode == OpCode.TRANSFER:
            to = self.stack.pop()
            from_account = self.stack.pop()
            amount = self.stack.pop()
            self.blockchain.transfer(from_account, to, amount)
        elif instruction.opcode == OpCode.VOTE:
            voter = self.stack.pop()
            proposal = self.stack.pop()
            vote = self.stack.pop()
            self.blockchain.vote(voter, proposal, vote)
        elif instruction.opcode == OpCode.GET_VOTE_RESULT:
            proposal = self.stack.pop()
            result = self.blockchain.get_vote_result(proposal)
            self.stack.append(result)