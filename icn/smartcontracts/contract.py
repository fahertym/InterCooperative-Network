# icn/blockchain/contract.py

import re

class SmartContract:
    def __init__(self, code):
        self.code = code
        self.parsed_code = self.parse(code)

    def parse(self, code):
        lines = code.split('\n')
        parsed = []
        for line in lines:
            line = line.strip()
            if line.startswith('transfer'):
                match = re.match(r'transfer\s+(\w+)\s+to\s+(\w+)\s+amount\s+(\d+)', line)
                if match:
                    parsed.append(('transfer', match.group(1), match.group(2), int(match.group(3))))
            elif line.startswith('vote'):
                match = re.match(r'vote\s+(\w+)\s+(\w+)', line)
                if match:
                    parsed.append(('vote', match.group(1), match.group(2)))
            elif line.startswith('if'):
                match = re.match(r'if\s+(\w+)\s+(\w+)\s+(\w+)', line)
                if match:
                    parsed.append(('if', match.group(1), match.group(2), match.group(3)))
            elif line == 'else':
                parsed.append(('else',))
            elif line == 'endif':
                parsed.append(('endif',))
        return parsed

    def validate(self):
        stack = []
        for op in self.parsed_code:
            if op[0] == 'if':
                stack.append('if')
            elif op[0] == 'else':
                if not stack or stack[-1] != 'if':
                    return False
            elif op[0] == 'endif':
                if not stack or stack[-1] != 'if':
                    return False
                stack.pop()
        return len(stack) == 0

class SmartContractParser:
    @staticmethod
    def parse(code):
        return SmartContract(code)