import time
from ..blockchain.transaction import Transaction

class Proposal:
    def __init__(self, id, creator_did, description, voting_period):
        self.id = id
        self.creator_did = creator_did
        self.description = description
        self.voting_period = voting_period
        self.start_time = time.time()
        self.votes = {}
        self.executed = False

    def is_active(self):
        return time.time() < self.start_time + self.voting_period

    def add_vote(self, voter_did, vote):
        if self.is_active():
            self.votes[voter_did] = vote
            return True
        return False

    def get_result(self):
        yes_votes = sum(1 for vote in self.votes.values() if vote)
        no_votes = len(self.votes) - yes_votes
        return yes_votes > no_votes

class DAO:
    def __init__(self, blockchain, name):
        self.blockchain = blockchain
        self.name = name
        self.members = set()
        self.proposals = {}
        self.next_proposal_id = 0

    def add_member(self, did):
        self.members.add(did)

    def remove_member(self, did):
        self.members.remove(did)

    def create_proposal(self, creator_did, description, voting_period):
        if creator_did not in self.members:
            return None
        proposal = Proposal(self.next_proposal_id, creator_did, description, voting_period)
        self.proposals[self.next_proposal_id] = proposal
        self.next_proposal_id += 1
        return proposal.id

    def vote_on_proposal(self, proposal_id, voter_did, vote):
        if voter_did not in self.members:
            return False
        proposal = self.proposals.get(proposal_id)
        if proposal:
            return proposal.add_vote(voter_did, vote)
        return False

    def execute_proposal(self, proposal_id):
        proposal = self.proposals.get(proposal_id)
        if proposal and not proposal.is_active() and not proposal.executed:
            if proposal.get_result():
                # Here you would implement the logic to execute the proposal
                # This could involve creating a transaction, updating DAO parameters, etc.
                proposal.executed = True
                return True
        return False

    def get_member_voting_power(self, did):
        # In a real implementation, this could be based on the member's stake or other factors
        return 1 if did in self.members else 0

class DAOManager:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.daos = {}

    def create_dao(self, name):
        if name in self.daos:
            return None
        dao = DAO(self.blockchain, name)
        self.daos[name] = dao
        return dao

    def get_dao(self, name):
        return self.daos.get(name)