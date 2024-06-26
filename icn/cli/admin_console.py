# icn/cli/admin_console.py

import cmd
import sys
from ..blockchain.chain import Blockchain
from ..dao.governance import VotingStrategy, ProposalType

class AdminConsole(cmd.Cmd):
    intro = "Welcome to the InterCooperative Network Admin Console. Type help or ? to list commands.\n"
    prompt = "(ICN Admin) "

    def __init__(self):
        super().__init__()
        self.blockchain = Blockchain()
        self.current_user = None

    def do_create_user(self, arg):
        """Create a new user (validator) with a specified stake: CREATE_USER <stake>"""
        try:
            stake = int(arg)
            did = self.blockchain.create_did()
            if self.blockchain.add_validator(did, stake):
                print(f"User created with DID: {did} and stake: {stake}")
                self.current_user = did
            else:
                print("Failed to create user. Stake might be too low.")
        except ValueError:
            print("Invalid stake value. Please enter an integer.")

    def do_switch_user(self, arg):
        """Switch to an existing user: SWITCH_USER <did>"""
        if arg in self.blockchain.consensus.validators:
            self.current_user = arg
            print(f"Switched to user with DID: {arg}")
        else:
            print("User not found.")

    def do_create_cooperative(self, arg):
        """Create a new cooperative: CREATE_COOPERATIVE <name>"""
        if not arg:
            print("Please provide a name for the cooperative.")
            return
        coop = self.blockchain.create_cooperative(arg)
        if coop:
            print(f"Cooperative '{arg}' created successfully.")
        else:
            print(f"Failed to create cooperative '{arg}'.")

    def do_add_member(self, arg):
        """Add a member to a cooperative: ADD_MEMBER <cooperative_name> <member_did>"""
        try:
            coop_name, member_did = arg.split()
            coop = self.blockchain.get_cooperative(coop_name)
            if coop:
                coop.add_member(member_did)
                print(f"Added {member_did} to cooperative '{coop_name}'.")
            else:
                print(f"Cooperative '{coop_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: ADD_MEMBER <cooperative_name> <member_did>")

    def do_create_proposal(self, arg):
        """Create a new proposal: CREATE_PROPOSAL <cooperative_name> <proposal_type> <description> <voting_period> <voting_strategy> [required_majority]"""
        args = arg.split()
        if len(args) < 5:
            print("Invalid input. Please use format: CREATE_PROPOSAL <cooperative_name> <proposal_type> <description> <voting_period> <voting_strategy> [required_majority]")
            return

        coop_name, proposal_type, description, voting_period, voting_strategy, *rest = args
        required_majority = float(rest[0]) if rest else 0.5

        coop = self.blockchain.get_cooperative(coop_name)
        if not coop:
            print(f"Cooperative '{coop_name}' not found.")
            return

        try:
            proposal_type = ProposalType[proposal_type.upper()]
            voting_strategy = VotingStrategy[voting_strategy.upper()]
            voting_period = int(voting_period)
        except (KeyError, ValueError):
            print("Invalid proposal type, voting strategy, or voting period.")
            return

        if not self.current_user:
            print("Please switch to a user first.")
            return

        proposal_id = coop.create_proposal(self.current_user, description, proposal_type, voting_period, voting_strategy, required_majority)
        if proposal_id is not None:
            print(f"Proposal created with ID: {proposal_id}")
        else:
            print("Failed to create proposal.")

    def do_vote(self, arg):
        """Vote on a proposal: VOTE <cooperative_name> <proposal_id> <yes/no>"""
        try:
            coop_name, proposal_id, vote = arg.split()
            coop = self.blockchain.get_cooperative(coop_name)
            if not coop:
                print(f"Cooperative '{coop_name}' not found.")
                return

            if not self.current_user:
                print("Please switch to a user first.")
                return

            vote_result = coop.vote_on_proposal(int(proposal_id), self.current_user, vote.lower() == 'yes')
            if vote_result:
                print("Vote cast successfully.")
            else:
                print("Failed to cast vote.")
        except ValueError:
            print("Invalid input. Please use format: VOTE <cooperative_name> <proposal_id> <yes/no>")

    def do_execute_proposal(self, arg):
        """Execute a proposal: EXECUTE_PROPOSAL <cooperative_name> <proposal_id>"""
        try:
            coop_name, proposal_id = arg.split()
            coop = self.blockchain.get_cooperative(coop_name)
            if not coop:
                print(f"Cooperative '{coop_name}' not found.")
                return

            result = coop.execute_proposal(int(proposal_id))
            if result:
                print("Proposal executed successfully.")
            else:
                print("Failed to execute proposal.")
        except ValueError:
            print("Invalid input. Please use format: EXECUTE_PROPOSAL <cooperative_name> <proposal_id>")

    def do_quit(self, arg):
        """Quit the admin console"""
        print("Thank you for using the InterCooperative Network Admin Console.")
        return True

if __name__ == '__main__':
    AdminConsole().cmdloop()