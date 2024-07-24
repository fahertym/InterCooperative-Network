// crates/icn_language/src/bytecode.rs

use crate::Statement;
use icn_vm::Opcode;

pub fn generate_bytecode(statements: &[Statement]) -> Vec<Opcode> {
    let mut bytecode = Vec::new();

    for statement in statements {
        match statement {
            Statement::Net_Node_Connect { node1, node2 } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(node1.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::String(node2.clone())));
                bytecode.push(Opcode::NetNodeConnect);
            },
            Statement::Chain_Block_Create { transactions } => {
                for tx in transactions {
                    bytecode.push(Opcode::Push(icn_vm::Value::String(tx.clone())));
                }
                bytecode.push(Opcode::Push(icn_vm::Value::Int(transactions.len() as i64)));
                bytecode.push(Opcode::ChainBlockCreate);
            },
            Statement::Econ_Currency_Mint { amount, currency_type } => {
                bytecode.push(Opcode::Push(icn_vm::Value::Float(*amount)));
                bytecode.push(Opcode::Push(icn_vm::Value::String(currency_type.clone())));
                bytecode.push(Opcode::EconCurrencyMint);
            },
            Statement::Gov_Proposal_Submit { description } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(description.clone())));
                bytecode.push(Opcode::GovProposalSubmit);
            },
            Statement::Coop_Member_Add { coop_id, member_id } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(coop_id.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::String(member_id.clone())));
                bytecode.push(Opcode::CoopMemberAdd);
            },
            Statement::Comm_Event_Organize { event_details } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(event_details.clone())));
                bytecode.push(Opcode::CommEventOrganize);
            },
            Statement::Vote_On_Proposal { proposal_id, vote } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(proposal_id.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::Bool(*vote)));
                bytecode.push(Opcode::VoteOnProposal);
            },
            Statement::Allocate_Resource { resource, amount } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(resource.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::Int(*amount)));
                bytecode.push(Opcode::AllocateResource);
            },
            Statement::Update_Reputation { address, change } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(address.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::Int(*change)));
                bytecode.push(Opcode::UpdateReputation);
            },
            Statement::Create_Proposal { title, description } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(title.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::String(description.clone())));
                bytecode.push(Opcode::CreateProposal);
            },
            Statement::Get_Proposal_Status { proposal_id } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(proposal_id.clone())));
                bytecode.push(Opcode::GetProposalStatus);
            },
            Statement::Emit_Event { event_name, event_data } => {
                bytecode.push(Opcode::Push(icn_vm::Value::String(event_name.clone())));
                bytecode.push(Opcode::Push(icn_vm::Value::String(event_data.clone())));
                bytecode.push(Opcode::EmitEvent);
            },
        }
    }

    bytecode
}