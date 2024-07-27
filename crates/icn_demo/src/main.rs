use icn_dao::{DaoFactory, DaoType, DaoTrait};
use chrono::Duration;

fn main() {
    println!("InterCooperative Network Demo");
    println!("==============================");

    // Create a Cooperative
    let mut coop = DaoFactory::create_dao(
        DaoType::Cooperative,
        "Green Energy Coop".to_string(),
        0.5,
        0.6
    );
    println!("Created a Cooperative: {}", coop.get_dao().name);

    // Add members to the Cooperative
    coop.add_member("alice".to_string(), "Alice".to_string()).unwrap();
    coop.add_member("bob".to_string(), "Bob".to_string()).unwrap();
    println!("Added members Alice and Bob to the Cooperative");

    // Create a proposal in the Cooperative
    let coop_proposal_id = coop.create_proposal(
        "Invest in Solar Panels".to_string(),
        "Proposal to invest in solar panel installation for our members".to_string(),
        "alice".to_string(),
        Duration::days(7)
    ).unwrap();
    println!("Created a proposal in the Cooperative: {}", coop_proposal_id);

    // Vote on the Cooperative proposal
    coop.vote(&coop_proposal_id, "alice", true).unwrap();
    coop.vote(&coop_proposal_id, "bob", true).unwrap();
    println!("Members voted on the Cooperative proposal");

    // Finalize and execute the Cooperative proposal
    let coop_status = coop.finalize_proposal(&coop_proposal_id).unwrap();
    println!("Cooperative proposal status: {:?}", coop_status);
    coop.execute_proposal(&coop_proposal_id).unwrap();
    println!("Executed the Cooperative proposal");

    println!("\n");

    // Create a Community
    let mut community = DaoFactory::create_dao(
        DaoType::Community,
        "Sustainable Living Community".to_string(),
        0.5,
        0.6
    );
    println!("Created a Community: {}", community.get_dao().name);

    // Add members to the Community
    community.add_member("carol".to_string(), "Carol".to_string()).unwrap();
    community.add_member("dave".to_string(), "Dave".to_string()).unwrap();
    println!("Added members Carol and Dave to the Community");

    // Create a proposal in the Community
    let community_proposal_id = community.create_proposal(
        "Community Garden".to_string(),
        "Proposal to create a community garden in the local park".to_string(),
        "carol".to_string(),
        Duration::days(7)
    ).unwrap();
    println!("Created a proposal in the Community: {}", community_proposal_id);

    // Vote on the Community proposal
    community.vote(&community_proposal_id, "carol", true).unwrap();
    community.vote(&community_proposal_id, "dave", true).unwrap();
    println!("Members voted on the Community proposal");

    // Finalize and execute the Community proposal
    let community_status = community.finalize_proposal(&community_proposal_id).unwrap();
    println!("Community proposal status: {:?}", community_status);
    community.execute_proposal(&community_proposal_id).unwrap();
    println!("Executed the Community proposal");

    println!("\nDemo completed successfully!");
}