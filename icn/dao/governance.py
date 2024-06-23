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

    def add_vote(self, voter, vote, voting_power):
        if self.is_active():
            self.votes[voter] = {'vote': vote, 'power': voting_power}
            return True
        return False

    def get_result(self):
        yes_votes = sum(v['power'] for v in self.votes.values() if v['vote'])
        total_votes = sum(v['power'] for v in self.votes.values())
        if total_votes == 0:
            return False
        return (yes_votes / total_votes) > self.required_majority

class DAO:
    def __init__(self, blockchain, name, token_address):
        self.blockchain = blockchain
        self.name = name
        self.token_address = token_address
        self.members = set()
        self.proposals = {}
        self.next_proposal_id = 0

    def add_member(self, did):
        self.members.add(did)

    def remove_member(self, did):
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
            voting_power = self.get_member_voting_power(voter)
            return proposal.add_vote(voter, vote, voting_power)
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
                    self.blockchain.add_transaction(Transaction(self.token_address, recipient, float(amount)))
                # Add more proposal types as needed
                proposal.executed = True
                return True
        return False

    def get_member_voting_power(self, member):
        # In a real implementation, this could be based on token balance or other factors
        return self.blockchain.get_balance(member)

class DAOManager:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.daos = {}

    def create_dao(self, name, token_address):
        if name in self.daos:
            return None
        dao = DAO(self.blockchain, name, token_address)
        self.daos[name] = dao
        return dao

    def get_dao(self, name):
        return self.daos.get(name)