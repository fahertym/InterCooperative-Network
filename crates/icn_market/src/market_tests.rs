use icn_market::{Market, Cooperative, Community, Member, Resource, Labor};
use std::collections::HashMap;

#[test]
fn test_trade_resource_between_cooperatives() {
    let mut market = Market::new();
    
    let mut coop1 = Cooperative {
        id: "coop1".to_string(),
        name: "Cooperative 1".to_string(),
        resources: HashMap::new(),
    };
    coop1.resources.insert("Wheat".to_string(), 100.0);
    
    let coop2 = Cooperative {
        id: "coop2".to_string(),
        name: "Cooperative 2".to_string(),
        resources: HashMap::new(),
    };
    
    market.add_cooperative(coop1);
    market.add_cooperative(coop2);
    
    let resource = Resource {
        name: "Wheat".to_string(),
        quantity: 50.0,
        unit: "kg".to_string(),
    };
    
    assert!(market.trade_resource("coop1", "coop2", resource).is_ok());
}

#[test]
fn test_exchange_labor_between_members() {
    let mut market = Market::new();
    
    let mut member1 = Member {
        id: "member1".to_string(),
        name: "Member 1".to_string(),
        skills: HashMap::new(),
    };
    member1.skills.insert("Carpentry".to_string(), 20.0);
    
    let member2 = Member {
        id: "member2".to_string(),
        name: "Member 2".to_string(),
        skills: HashMap::new(),
    };
    
    market.add_member(member1);
    market.add_member(member2);
    
    let labor = Labor {
        skill: "Carpentry".to_string(),
        hours: 10.0,
    };
    
    assert!(market.exchange_labor("member1", "member2", labor).is_ok());
}
