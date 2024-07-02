# icn/federation/federation.py

from ..smartcontracts.language import SmartContractLanguage
from ..smartcontracts.vm import SmartContractVM

class Federation:
    def __init__(self, name, member_cooperatives, blockchain):
        self.name = name
        self.member_cooperatives = set(member_cooperatives)
        self.shared_resources = {}
        self.inter_coop_agreements = {}
        self.contracts = {}
        self.blockchain = blockchain
        self.vm = SmartContractVM(blockchain)

    def add_cooperative(self, cooperative):
        self.member_cooperatives.add(cooperative)

    def remove_cooperative(self, cooperative):
        self.member_cooperatives.remove(cooperative)

    def get_members(self):
        return list(self.member_cooperatives)

    def add_shared_resource(self, resource_name, resource_data):
        self.shared_resources[resource_name] = resource_data

    def get_shared_resource(self, resource_name):
        return self.shared_resources.get(resource_name)

    def create_agreement(self, coop1, coop2, agreement_terms):
        if coop1 in self.member_cooperatives and coop2 in self.member_cooperatives:
            agreement_id = f"{coop1.name}_{coop2.name}_{len(self.inter_coop_agreements)}"
            self.inter_coop_agreements[agreement_id] = {
                "coop1": coop1,
                "coop2": coop2,
                "terms": agreement_terms
            }
            return agreement_id
        return None

    def get_agreement(self, agreement_id):
        return self.inter_coop_agreements.get(agreement_id)

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

class FederationManager:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.federations = {}
        self.inter_federation_agreements = {}

    def create_federation(self, name, member_cooperatives):
        if name not in self.federations:
            federation = Federation(name, member_cooperatives, self.blockchain)
            self.federations[name] = federation
            return federation
        return None

    def get_federation(self, name):
        return self.federations.get(name)

    def list_federations(self):
        return list(self.federations.keys())

    def add_cooperative_to_federation(self, federation_name, cooperative):
        federation = self.get_federation(federation_name)
        if federation:
            federation.add_cooperative(cooperative)
            return True
        return False

    def remove_cooperative_from_federation(self, federation_name, cooperative):
        federation = self.get_federation(federation_name)
        if federation:
            federation.remove_cooperative(cooperative)
            return True
        return False

    def create_inter_federation_agreement(self, federation1_name, federation2_name, agreement_terms):
        federation1 = self.get_federation(federation1_name)
        federation2 = self.get_federation(federation2_name)
        if federation1 and federation2:
            agreement_id = f"{federation1_name}_{federation2_name}_{len(self.inter_federation_agreements)}"
            self.inter_federation_agreements[agreement_id] = {
                "federation1": federation1,
                "federation2": federation2,
                "terms": agreement_terms
            }
            return agreement_id
        return None

    def get_inter_federation_agreement(self, agreement_id):
        return self.inter_federation_agreements.get(agreement_id)

    def deploy_federation_contract(self, federation_name, contract_code):
        federation = self.get_federation(federation_name)
        if federation:
            return federation.deploy_contract(contract_code)
        return None

    def execute_federation_contract(self, federation_name, contract_id, *args):
        federation = self.get_federation(federation_name)
        if federation:
            return federation.execute_contract(contract_id, *args)
        return False