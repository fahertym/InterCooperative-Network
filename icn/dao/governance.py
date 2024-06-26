import time

class Proposal:
    def __init__(self, id, creator, description, proposal_type, voting_period, required_majority=0.5):
        self.id = id
        self.creator = creator
        self.description = description
        self.proposal_type = proposal_type
        self.voting_period = voting_period
        self.required_majority = required_majority
        self.start_time = time.time()
        self.votes = {}
        self.executed = False

    def is_active(self):
        return time.time() < self.start_time + self.voting_period

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
        return (yes_votes / total_votes) > self.required_majority

class Cooperative:
    def __init__(self, blockchain, name):
        self.blockchain = blockchain
        self.name = name
        self.members = set()
        self.proposals = {}
        self.next_proposal_id = 0

    def add_member(self, did):
        self.members.add(did)

    def remove_member(self, did):
        if did in self.members:
            self.members.remove(did)

    def create_proposal(self, creator, description, proposal_type, voting_period, required_majority=0.5):
        if creator not in self.members:
            return None
        proposal = Proposal(self.next_proposal_id, creator, description, proposal_type, voting_period, required_majority)
        self.proposals[self.next_proposal_id] = proposal
        self.next_proposal_id += 1
        return proposal.id

    def vote_on_proposal(self, proposal_id, voter, vote):
        if voter not in self.members:
            return False
        proposal = self.proposals.get(proposal_id)
        if proposal:
            return proposal.add_vote(voter, vote)
        return False

    def execute_proposal(self, proposal_id):
        proposal = self.proposals.get(proposal_id)
        if proposal and not proposal.is_active() and not proposal.executed:
            if proposal.get_result():
                if proposal.proposal_type == "add_member":
                    self.add_member(proposal.description)
                elif proposal.proposal_type == "remove_member":
                    self.remove_member(proposal.description)
                elif proposal.proposal_type == "transfer_funds":
                    recipient, amount = proposal.description.split(',')
                    amount = float(amount)
                    tx = self.blockchain.create_transaction(self.name, recipient, amount)
                    self.blockchain.add_transaction(tx)
                proposal.executed = True
                return True
        return False

    def get_member_info(self, did):
        if did in self.members:
            balance = self.blockchain.get_balance(did)
            return {
                "did": did,
                "balance": balance,
                "is_validator": self.blockchain.consensus.is_validator(did)
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
                "members": len(coop.members),
                "proposals": len(coop.proposals),
                "active_proposals": sum(1 for p in coop.proposals.values() if p.is_active())
            }
        return None