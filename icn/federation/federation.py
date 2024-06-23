# icn/federation.py

class Federation:
    def __init__(self, name, member_daos):
        self.name = name
        self.member_daos = set(member_daos)

    def add_dao(self, dao):
        self.member_daos.add(dao)

    def remove_dao(self, dao):
        self.member_daos.remove(dao)

    def get_members(self):
        return list(self.member_daos)

class FederationManager:
    def __init__(self):
        self.federations = {}

    def create_federation(self, name, member_daos):
        if name not in self.federations:
            federation = Federation(name, member_daos)
            self.federations[name] = federation
            return federation
        return None

    def get_federation(self, name):
        return self.federations.get(name)

    def list_federations(self):
        return list(self.federations.keys())

    def add_dao_to_federation(self, federation_name, dao):
        federation = self.get_federation(federation_name)
        if federation:
            federation.add_dao(dao)
            return True
        return False

    def remove_dao_from_federation(self, federation_name, dao):
        federation = self.get_federation(federation_name)
        if federation:
            federation.remove_dao(dao)
            return True
        return False