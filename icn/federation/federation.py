# icn/federation/federation.py

class Federation:
    def __init__(self, name, member_cooperatives):
        self.name = name
        self.member_cooperatives = set(member_cooperatives)
        self.shared_resources = {}
        self.inter_coop_agreements = {}

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

class FederationManager:
    def __init__(self):
        self.federations = {}
        self.inter_federation_agreements = {}

    def create_federation(self, name, member_cooperatives):
        if name not in self.federations:
            federation = Federation(name, member_cooperatives)
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