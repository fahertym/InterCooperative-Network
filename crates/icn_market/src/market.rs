use crate::entities::{Cooperative, Community, Member, Resource, Labor};
use crate::transaction::{Transaction, TransactionType, TransactionResult};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Market {
    pub cooperatives: HashMap<String, Cooperative>,
    pub communities: HashMap<String, Community>,
    pub members: HashMap<String, Member>,
    pub transactions: Vec<Transaction>,
}

impl Market {
    pub fn new() -> Self {
        Market {
            cooperatives: HashMap::new(),
            communities: HashMap::new(),
            members: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    pub fn add_cooperative(&mut self, cooperative: Cooperative) {
        self.cooperatives.insert(cooperative.id.clone(), cooperative);
    }

    pub fn add_community(&mut self, community: Community) {
        self.communities.insert(community.id.clone(), community);
    }

    pub fn add_member(&mut self, member: Member) {
        self.members.insert(member.id.clone(), member);
    }

    pub fn trade_resource(&mut self, from_id: &str, to_id: &str, resource: Resource) -> TransactionResult {
        let transaction = Transaction::new(
            TransactionType::ResourceTrade,
            from_id.to_string(),
            to_id.to_string(),
            Some(resource.clone()),
            None,
        );

        if let Some(from_coop) = self.cooperatives.get_mut(from_id) {
            if let Some(to_coop) = self.cooperatives.get_mut(to_id) {
                from_coop.resources.entry(resource.name.clone()).and_modify(|e| *e -= resource.quantity);
                to_coop.resources.entry(resource.name.clone()).and_modify(|e| *e += resource.quantity).or_insert(resource.quantity);
                self.transactions.push(transaction);
                return Ok(());
            }
        }

        if let Some(from_comm) = self.communities.get_mut(from_id) {
            if let Some(to_comm) = self.communities.get_mut(to_id) {
                from_comm.members.get_mut(&resource.name).unwrap().skills.entry(resource.name.clone()).and_modify(|e| *e -= resource.quantity);
                to_comm.members.get_mut(&resource.name).unwrap().skills.entry(resource.name.clone()).and_modify(|e| *e += resource.quantity).or_insert(resource.quantity);
                self.transactions.push(transaction);
                return Ok(());
            }
        }

        if let Some(from_mem) = self.members.get_mut(from_id) {
            if let Some(to_mem) = self.members.get_mut(to_id) {
                from_mem.skills.entry(resource.name.clone()).and_modify(|e| *e -= resource.quantity);
                to_mem.skills.entry(resource.name.clone()).and_modify(|e| *e += resource.quantity).or_insert(resource.quantity);
                self.transactions.push(transaction);
                return Ok(());
            }
        }

        Err("Invalid trade".into())
    }

    pub fn exchange_labor(&mut self, from_id: &str, to_id: &str, labor: Labor) -> TransactionResult {
        let transaction = Transaction::new(
            TransactionType::LaborExchange,
            from_id.to_string(),
            to_id.to_string(),
            None,
            Some(labor.clone()),
        );

        if let Some(from_mem) = self.members.get_mut(from_id) {
            if let Some(to_mem) = self.members.get_mut(to_id) {
                from_mem.skills.entry(labor.skill.clone()).and_modify(|e| *e -= labor.hours);
                to_mem.skills.entry(labor.skill.clone()).and_modify(|e| *e += labor.hours).or_insert(labor.hours);
                self.transactions.push(transaction);
                return Ok(());
            }
        }

        Err("Invalid exchange".into())
    }
}

impl Default for Market {
    fn default() -> Self {
        Self::new()
    }
}
