# icn/dao/governance.py

import time
from enum import Enum
from ..smartcontracts.language import SmartContractLanguage
from ..smartcontracts.vm import SmartContractVM

class VotingStrategy(Enum):
    SIMPLE_MAJORITY = 1
    SUPER_MAJORITY = 2
    CONSENSUS = 3

class ProposalType(Enum):
    ADD_MEMBER = 1
    REMOVE_MEMBER = 2
    CHANGE_RULES = 3
    ALLOCATE_FUNDS = 4
    CHANGE_LEADERSHIP = 5
    DEPLOY_CONTRACT = 6

class ProposalStatus(Enum):
    ACTIVE = 1
    PASSED = 2
    FAILED = 3
    EXECUTED = 4

class Proposal:
    def __init__(self, id, creator, description, proposal_type, voting_period, voting_strategy, required_majority=0.5, contract_code=None):
        self.id = id
        self.creator = creator
        self.description = description
        self.proposal_type = proposal_type
        self.voting_period = voting_period
        self.voting_strategy = voting_strategy
        self.required_majority = required_majority
        self.start_time = time.time()
        self.votes = {}
        self.status = ProposalStatus.ACTIVE
        self.contract_code = contract_code

    def is_active(self):
        return time.time() < self.start_time + self.voting_period and self.status == ProposalStatus.ACTIVE

    def add_vote(self, voter, vote):
        if self.is_active():
            self.votes[voter] = vote
            return True
        return False

    def get_result(self):
        yes_votes = sum(1 for vote in self.votes.values() if vote)
        total_votes = len(self.votes)
        if total_votes == 0:
            return False
        
        if self.voting_strategy == VotingStrategy.SIMPLE_MAJORITY:
            return (yes_votes / total_votes) > 0.5
        elif self.voting_strategy == VotingStrategy.SUPER_MAJORITY:
            return (yes_votes / total_votes) >= self.required_majority
        elif self.voting_strategy == VotingStrategy.CONSENSUS:
            return yes_votes == total_votes
        
        return False

    def finalize(self):
        if not self.is_active():
            if self.get_result():
                self.status = ProposalStatus.PASSED
            else:
                self.status = ProposalStatus.FAILED

class Cooperative:
    def __init__(self, blockchain, name):
        self.blockchain = blockchain
        self.name = name
        self.did = self.blockchain.create_did()
        self.members = set()
        self.admin_members = set()
        self.proposals = {}
        self.next_proposal_id = 0
        self.leadership = set()
        self.contracts = {}
        self.vm = SmartContractVM(blockchain)

    def add_member(self, did, is_admin=False):
        self.members.add(did)
        if is_admin:
            self.admin_members.add(did)

    def remove_member(self, did):
        if did in self.members:
            self.members.remove(did)
        if did in self.admin_members:
            self.admin_members.remove(did)
        if did in self.leadership:
            self.leadership.remove(did)

    def is_admin(self, did):
        return did in self.admin_members

    def create_proposal(self, creator, description, proposal_type, voting_period, voting_strategy="SIMPLE_MAJORITY", required_majority=0.5, contract_code=None):
        if creator not in self.members:
            return None
        proposal = Proposal(self.next_proposal_id, creator, description, ProposalType[proposal_type], voting_period, VotingStrategy[voting_strategy], required_majority, contract_code)
        self.proposals[self.next_proposal_id] = proposal
        self.next_proposal_id += 1
        return proposal.id

    def vote_on_proposal(self, proposal_id, voter, vote):
        if voter not in self.members:
            return False
        proposal = self.proposals.get(proposal_id)
        if proposal:
            result = proposal.add_vote(voter, vote)
            if result:
                tx = self.blockchain.create_transaction(voter, self.did, 0, message=f"Vote on proposal {proposal_id}")
                self.blockchain.add_transaction(tx)
            return result
        return False

    def execute_proposal(self, proposal_id):
        proposal = self.proposals.get(proposal_id)
        if proposal and not proposal.is_active():
            proposal.finalize()
            if proposal.status == ProposalStatus.PASSED:
                if proposal.proposal_type == ProposalType.ADD_MEMBER:
                    self.add_member(proposal.description)
                elif proposal.proposal_type == ProposalType.REMOVE_MEMBER:
                    self.remove_member(proposal.description)
                elif proposal.proposal_type == ProposalType.ALLOCATE_FUNDS:
                    recipient, amount = proposal.description.split(',')
                    amount = float(amount)
                    tx = self.blockchain.create_transaction(self.did, recipient, amount)
                    self.blockchain.add_transaction(tx)
                elif proposal.proposal_type == ProposalType.CHANGE_LEADERSHIP:
                    new_leadership = set(proposal.description.split(','))
                    self.leadership = new_leadership
                elif proposal.proposal_type == ProposalType.CHANGE_RULES:
                    print(f"Rules changed for cooperative {self.name}: {proposal.description}")
                elif proposal.proposal_type == ProposalType.DEPLOY_CONTRACT:
                    if proposal.contract_code:
                        self.deploy_contract(proposal.contract_code)
                
                proposal.status = ProposalStatus.EXECUTED
                tx = self.blockchain.create_transaction(self.did, self.did, 0, message=f"Execute proposal {proposal_id}")
                self.blockchain.add_transaction(tx)
                return True
        return False

    def deploy_contract(self, contract_code):
        contract_id = f"contract_{len(self.contracts)}"
        instructions = SmartContractLanguage.parse(contract_code)
        if SmartContractLanguage.validate(instructions):
            self.contracts[contract_id] = instructions
            return contract_id
        return None

    def execute_contract(self, contract_id, *args):
        instructions = self.contracts.get(contract_id)
        if instructions:
            self.vm.execute(instructions)
            return True
        return False

    def get_member_info(self, did):
        if did in self.members:
            balance = self.blockchain.get_balance(did)
            return {
                "did": did,
                "balance": balance,
                "is_validator": self.blockchain.consensus.is_validator(did),
                "is_leader": did in self.leadership
            }
        return None

class CooperativeManager:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.cooperatives = {}

    def create_cooperative(self, name):
        if name in self.cooperatives:
            return None
        cooperative = Cooperative(self.blockchain, name)
        self.cooperatives[name] = cooperative
        return cooperative

    def get_cooperative(self, name):
        return self.cooperatives.get(name)

    def list_cooperatives(self):
        return list(self.cooperatives.keys())

    def get_cooperative_info(self, name):
        coop = self.get_cooperative(name)
        if coop:
            return {
                "name": coop.name,
                "did": coop.did,
                "members": len(coop.members),
                "proposals": len(coop.proposals),
                "active_proposals": sum(1 for p in coop.proposals.values() if p.is_active()),
                "leadership": list(coop.leadership),
                "contracts": len(coop.contracts)
            }
        return None