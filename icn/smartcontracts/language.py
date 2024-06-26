# icn/smartcontracts/language.py

from enum import Enum

class OpCode(Enum):
    PUSH = 1
    POP = 2
    ADD = 3
    SUB = 4
    MUL = 5
    DIV = 6
    STORE = 7
    LOAD = 8
    EQ = 9
    LT = 10
    GT = 11
    JMP = 12
    JMPIF = 13
    SET_PRICE = 14
    GET_PRICE = 15
    TRADE = 16
    GET_BALANCE = 17
    TRANSFER = 18
    VOTE = 19
    GET_VOTE_RESULT = 20

class Instruction:
    def __init__(self, opcode, args=None):
        self.opcode = opcode
        self.args = args if args is not None else []

class SmartContractLanguage:
    @staticmethod
    def parse(code):
        instructions = []
        lines = code.split('\n')
        for line in lines:
            parts = line.strip().split()
            if not parts:
                continue
            opcode = OpCode[parts[0].upper()]
            args = parts[1:] if len(parts) > 1 else []
            instructions.append(Instruction(opcode, args))
        return instructions