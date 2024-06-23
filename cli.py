# cli.py

import cmd
import sys
from icn.blockchain.chain import Blockchain
from icn.blockchain.transaction import Transaction

class ICNCLI(cmd.Cmd):
    intro = "Welcome to the InterCooperative Network CLI. Type help or ? to list commands.\n"
    prompt = "(ICN) "

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

    def do_create_transaction(self, arg):
        """Create a new transaction: CREATE_TRANSACTION <recipient> <amount>"""
        if not self.current_user:
            print("Please create or switch to a user first.")
            return
        try:
            recipient, amount = arg.split()
            amount = float(amount)
            tx = Transaction(self.current_user, recipient, amount)
            tx.sign_transaction(self.blockchain.did_manager)
            if self.blockchain.add_transaction(tx):
                print("Transaction created successfully.")
            else:
                print("Failed to create transaction.")
        except ValueError:
            print("Invalid input. Please use format: CREATE_TRANSACTION <recipient> <amount>")

    def do_mine_block(self, arg):
        """Mine a new block"""
        if not self.current_user:
            print("Please create or switch to a user first.")
            return
        if self.blockchain.mine_pending_transactions(self.current_user):
            print("Block mined successfully.")
        else:
            print("Failed to mine block.")

    def do_get_balance(self, arg):
        """Get the balance of a user: GET_BALANCE [did]"""
        did = arg if arg else self.current_user
        if did:
            balance = self.blockchain.get_balance(did)
            print(f"Balance of {did}: {balance}")
        else:
            print("Please specify a DID or switch to a user.")

    def do_get_validator_info(self, arg):
        """Get information about a validator: GET_VALIDATOR_INFO [did]"""
        did = arg if arg else self.current_user
        if did:
            info = self.blockchain.get_validator_info(did)
            if info:
                print(f"Validator {did} info:")
                print(f"  Stake: {info['stake']}")
                print(f"  Cooperation score: {info['cooperation_score']:.2f}")
                print(f"  Blocks created: {info['blocks_created']}")
                print(f"  Total uptime: {info['total_uptime']:.2f} seconds")
            else:
                print("Validator not found.")
        else:
            print("Please specify a DID or switch to a user.")

    def do_create_dao(self, arg):
        """Create a new DAO: CREATE_DAO <name>"""
        if not arg:
            print("Please specify a name for the DAO.")
            return
        dao = self.blockchain.create_dao(arg)
        if dao:
            print(f"DAO '{arg}' created successfully.")
        else:
            print(f"Failed to create DAO '{arg}'.")

    def do_add_dao_member(self, arg):
        """Add a member to a DAO: ADD_DAO_MEMBER <dao_name> <member_did>"""
        try:
            dao_name, member_did = arg.split()
            dao = self.blockchain.get_dao(dao_name)
            if dao:
                dao.add_member(member_did)
                print(f"Added {member_did} to DAO '{dao_name}'.")
            else:
                print(f"DAO '{dao_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: ADD_DAO_MEMBER <dao_name> <member_did>")

    def do_create_proposal(self, arg):
        """Create a new proposal in a DAO: CREATE_PROPOSAL <dao_name> <proposal_type> <description>"""
        if not self.current_user:
            print("Please create or switch to a user first.")
            return
        try:
            dao_name, proposal_type, description = arg.split(maxsplit=2)
            dao = self.blockchain.get_dao(dao_name)
            if dao:
                proposal_id = dao.create_proposal(self.current_user, description, proposal_type, 3600)
                if proposal_id is not None:
                    print(f"Proposal created with ID: {proposal_id}")
                else:
                    print("Failed to create proposal.")
            else:
                print(f"DAO '{dao_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: CREATE_PROPOSAL <dao_name> <proposal_type> <description>")

    def do_vote_proposal(self, arg):
        """Vote on a proposal: VOTE_PROPOSAL <dao_name> <proposal_id> <yes/no>"""
        if not self.current_user:
            print("Please create or switch to a user first.")
            return
        try:
            dao_name, proposal_id, vote = arg.split()
            dao = self.blockchain.get_dao(dao_name)
            if dao:
                vote_result = dao.vote_on_proposal(int(proposal_id), self.current_user, vote.lower() == 'yes')
                if vote_result:
                    print("Vote cast successfully.")
                else:
                    print("Failed to cast vote.")
            else:
                print(f"DAO '{dao_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: VOTE_PROPOSAL <dao_name> <proposal_id> <yes/no>")

    def do_execute_proposal(self, arg):
        """Execute a proposal: EXECUTE_PROPOSAL <dao_name> <proposal_id>"""
        try:
            dao_name, proposal_id = arg.split()
            dao = self.blockchain.get_dao(dao_name)
            if dao:
                result = dao.execute_proposal(int(proposal_id))
                if result:
                    print("Proposal executed successfully.")
                else:
                    print("Failed to execute proposal.")
            else:
                print(f"DAO '{dao_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: EXECUTE_PROPOSAL <dao_name> <proposal_id>")

    def do_quit(self, arg):
        """Quit the CLI"""
        print("Thank you for using the InterCooperative Network CLI.")
        return True

if __name__ == '__main__':
    ICNCLI().cmdloop()