async fn create_identity(node: &IcnNode) -> IcnResult<()> {
    println!("Creating a new identity...");
    
    print!("Enter name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), name.trim().to_string());
    
    let identity = node.create_identity(attributes).await?;
    println!("Identity created successfully. ID: {}", identity.id);
    Ok(())
}

async fn process_transaction(node: &IcnNode) -> IcnResult<()> {
    println!("Processing a new transaction...");
    
    print!("From (identity ID): ");
    io::stdout().flush()?;
    let mut from = String::new();
    io::stdin().read_line(&mut from)?;
    
    print!("To (identity ID): ");
    io::stdout().flush()?;
    let mut to = String::new();
    io::stdin().read_line(&mut to)?;
    
    print!("Amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: f64 = amount_str.trim().parse().map_err(|_| icn_common::IcnError::CustomError("Invalid amount".to_string()))?;

    let transaction = Transaction {
        from: from.trim().to_string(),
        to: to.trim().to_string(),
        amount,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None, // In a real scenario, this should be signed
    };

    node.process_transaction(transaction).await?;
    println!("Transaction processed successfully");
    Ok(())
}

async fn create_proposal(node: &IcnNode) -> IcnResult<()> {
    println!("Creating a new proposal...");
    
    print!("Title: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;
    
    print!("Description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    
    print!("Proposer (identity ID): ");
    io::stdout().flush()?;
    let mut proposer = String::new();
    io::stdin().read_line(&mut proposer)?;

    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        proposer: proposer.trim().to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Economic,
        required_quorum: 0.66,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal).await?;
    println!("Proposal created successfully. ID: {}", proposal_id);
    Ok(())
}

async fn vote_on_proposal(node: &IcnNode) -> IcnResult<()> {
    println!("Voting on a proposal...");
    
    print!("Proposal ID: ");
    io::stdout().flush()?;
    let mut proposal_id = String::new();
    io::stdin().read_line(&mut proposal_id)?;
    
    print!("Voter (identity ID): ");
    io::stdout().flush()?;
    let mut voter = String::new();
    io::stdin().read_line(&mut voter)?;
    
    print!("In favor? (yes/no): ");
    io::stdout().flush()?;
    let mut in_favor_str = String::new();
    io::stdin().read_line(&mut in_favor_str)?;
    let in_favor = in_favor_str.trim().to_lowercase() == "yes";

    node.vote_on_proposal(&proposal_id.trim(), voter.trim().to_string(), in_favor, 1.0).await?;
    println!("Vote recorded successfully");
    Ok(())
}

async fn check_balance(node: &IcnNode) -> IcnResult<()> {
    println!("Checking balance...");
    
    print!("Identity ID: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    
    let balance = node.get_balance(address.trim(), &CurrencyType::BasicNeeds).await?;
    println!("Balance: {} BasicNeeds", balance);
    Ok(())
}

async fn mint_currency(node: &IcnNode) -> IcnResult<()> {
    println!("Minting new currency...");
    
    print!("Identity ID: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    
    print!("Amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: f64 = amount_str.trim().parse().map_err(|_| icn_common::IcnError::CustomError("Invalid amount".to_string()))?;

    node.mint_currency(address.trim(), &CurrencyType::BasicNeeds, amount).await?;
    println!("Currency minted successfully");
    Ok(())
}

async fn get_network_stats(node: &IcnNode) -> IcnResult<()> {
    println!("Fetching network statistics...");
    
    let stats = node.get_network_stats().await?;
    println!("Network Statistics:");
    println!("  Node count: {}", stats.node_count);
    println!("  Total transactions: {}", stats.total_transactions);
    println!("  Active proposals: {}", stats.active_proposals);
    Ok(())
}