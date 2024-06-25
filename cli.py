import cmd
import sys
import time
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

    def do_list_daos(self, arg):
        """List all DAOs"""
        daos = self.blockchain.dao_manager.daos
        if daos:
            print("List of DAOs:")
            for name in daos:
                print(f"- {name}")
        else:
            print("No DAOs found.")

    def do_list_dao_members(self, arg):
        """List members of a DAO: LIST_DAO_MEMBERS <dao_name>"""
        if not arg:
            print("Please specify a DAO name.")
            return
        dao = self.blockchain.get_dao(arg)
        if dao:
            print(f"Members of DAO '{arg}':")
            for member in dao.members:
                print(f"- {member}")
        else:
            print(f"DAO '{arg}' not found.")

    def do_get_member_details(self, arg):
        """Get details of a DAO member: GET_MEMBER_DETAILS <dao_name> <member_did>"""
        try:
            dao_name, member_did = arg.split()
            dao = self.blockchain.get_dao(dao_name)
            if dao:
                if member_did in dao.members:
                    balance = self.blockchain.get_balance(member_did)
                    validator_info = self.blockchain.get_validator_info(member_did)
                    print(f"Member details for {member_did} in DAO '{dao_name}':")
                    print(f"  Balance: {balance}")
                    if validator_info:
                        print(f"  Stake: {validator_info['stake']}")
                        print(f"  Cooperation score: {validator_info['cooperation_score']:.2f}")
                        print(f"  Blocks created: {validator_info['blocks_created']}")
                        print(f"  Total uptime: {validator_info['total_uptime']:.2f} seconds")
                    else:
                        print("  Not a validator")
                else:
                    print(f"{member_did} is not a member of DAO '{dao_name}'")
            else:
                print(f"DAO '{dao_name}' not found.")
        except ValueError:
            print("Invalid input. Please use format: GET_MEMBER_DETAILS <dao_name> <member_did>")

    def do_run_test(self, arg):
        """Run the blockchain test"""
        from test_blockchain import test_blockchain
        test_blockchain()

    def do_network_health(self, arg):
        """Display network health information"""
        print("Network Health:")
        print(f"  Total blocks: {len(self.blockchain.chain)}")
        print(f"  Total transactions: {sum(len(block.transactions) for block in self.blockchain.chain)}")
        print(f"  Total validators: {len(self.blockchain.consensus.validators)}")
        print(f"  Total DAOs: {len(self.blockchain.dao_manager.daos)}")
        print(f"  Current difficulty: {self.blockchain.difficulty}")
        print(f"  Last block time: {time.ctime(self.blockchain.get_latest_block().timestamp)}")

    def do_blockchain_info(self, arg):
        """Display detailed blockchain information"""
        print("Blockchain Information:")
        print(f"  Chain length: {len(self.blockchain.chain)}")
        print(f"  Latest block hash: {self.blockchain.get_latest_block().hash}")
        print(f"  Total transactions: {sum(len(block.transactions) for block in self.blockchain.chain)}")
        print(f"  Pending transactions: {len(self.blockchain.pending_transactions)}")
        print(f"  Is chain valid: {self.blockchain.is_chain_valid()}")

    def do_list_cooperatives(self, arg):
        """List all cooperatives (DAOs)"""
        self.do_list_daos(arg)

    def do_cooperative_info(self, arg):
        """Display detailed information about a cooperative (DAO): COOPERATIVE_INFO <dao_name>"""
        if not arg:
            print("Please specify a cooperative (DAO) name.")
            return
        dao = self.blockchain.get_dao(arg)
        if dao:
            print(f"Cooperative (DAO) Information for '{arg}':")
            print(f"  Total members: {len(dao.members)}")
            print(f"  Total proposals: {len(dao.proposals)}")
            print("  Active proposals:")
            for proposal_id, proposal in dao.proposals.items():
                if proposal.is_active():
                    print(f"    - Proposal {proposal_id}: {proposal.description}")
        else:
            print(f"Cooperative (DAO) '{arg}' not found.")

    def do_create_federation(self, arg):
        """Create a new federation: CREATE_FEDERATION <federation_name> <dao1_name> <dao2_name> ..."""
        args = arg.split()
        if len(args) < 3:
            print("Please provide a federation name and at least two DAO names.")
            return
        federation_name = args[0]
        dao_names = args[1:]
        federation = self.blockchain.create_federation(federation_name, dao_names)
        if federation:
            print(f"Federation '{federation_name}' created with DAOs: {', '.join(dao_names)}")
        else:
            print("Failed to create federation. Please check the DAO names and try again.")

    def do_list_federations(self, arg):
        """List all federations"""
        federations = self.blockchain.list_federations()
        if federations:
            print("List of Federations:")
            for name in federations:
                print(f"- {name}")
        else:
            print("No federations found.")

    def do_federation_info(self, arg):
        """Display information about a federation: FEDERATION_INFO <federation_name>"""
        if not arg:
            print("Please specify a federation name.")
            return
        federation = self.blockchain.get_federation(arg)
        if federation:
            print(f"Federation Information for '{arg}':")
            print(f"  Total member DAOs: {len(federation.member_daos)}")
            print("  Member DAOs:")
            for dao in federation.get_members():
                print(f"    - {dao.name}")
        else:
            print(f"Federation '{arg}' not found.")

    def do_add_dao_to_federation(self, arg):
        """Add a DAO to a federation: ADD_DAO_TO_FEDERATION <federation_name> <dao_name>"""
        try:
            federation_name, dao_name = arg.split()
            if self.blockchain.add_dao_to_federation(federation_name, dao_name):
                print(f"Added DAO '{dao_name}' to federation '{federation_name}'.")
            else:
                print("Failed to add DAO to federation.")
        except ValueError:
            print("Invalid input. Please use format: ADD_DAO_TO_FEDERATION <federation_name> <dao_name>")

    def do_remove_dao_from_federation(self, arg):
        """Remove a DAO from a federation: REMOVE_DAO_FROM_FEDERATION <federation_name> <dao_name>"""
        try:
            federation_name, dao_name = arg.split()
            if self.blockchain.remove_dao_from_federation(federation_name, dao_name):
                print(f"Removed DAO '{dao_name}' from federation '{federation_name}'.")
            else:
                print("Failed to remove DAO from federation.")
        except ValueError:
            print("Invalid input. Please use format: REMOVE_DAO_FROM_FEDERATION <federation_name> <dao_name>")

    def do_deploy_contract(self, arg):
        """Deploy a new smart contract: DEPLOY_CONTRACT <code>"""
        if not arg:
            print("Please provide the contract code.")
            return
        contract_id = self.blockchain.deploy_contract(arg)
        if contract_id:
            print(f"Contract deployed successfully. Contract ID: {contract_id}")
        else:
            print("Failed to deploy contract.")

    def do_execute_contract(self, arg):
        """Execute a smart contract: EXECUTE_CONTRACT <contract_id> [args...]"""
        args = arg.split()
        if len(args) < 1:
            print("Please provide the contract ID and any necessary arguments.")
            return
        contract_id = args[0]
        result = self.blockchain.execute_contract(contract_id, *args[1:])
        print(f"Contract execution result: {result}")

    def do_quit(self, arg):
        """Quit the CLI"""
        print("Thank you for using the InterCooperative Network CLI.")
        return True

if __name__ == '__main__':
    ICNCLI().cmdloop()