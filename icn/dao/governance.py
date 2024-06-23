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
                # Execute the proposal based on its type
                if proposal.proposal_type == "add_member":
                    self.add_member(proposal.description)
                elif proposal.proposal_type == "remove_member":
                    self.remove_member(proposal.description)
                elif proposal.proposal_type == "transfer_funds":
                    recipient, amount = proposal.description.split(',')
                    # Note: In a real implementation, you'd need to handle this transaction properly
                    print(f"Transfer {amount} to {recipient}")
                # Add more proposal types as needed
                proposal.executed = True
                return True
        return False

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