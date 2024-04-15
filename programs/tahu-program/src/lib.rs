use anchor_lang::prelude::*;

declare_id!("DaoVotingProgram");

#[program]
pub mod dao_voting {
    use super::*;

    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal: Proposal,
    ) -> Result<()> {
        let proposal_account = &mut ctx.accounts.proposal_account;
        *proposal_account = proposal;
        Ok(())
    }

    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>,
        vote: Vote,
    ) -> Result<()> {
        let proposal_account = &mut ctx.accounts.proposal_account;
        match vote.vote_type {
            VoteType::Yes => proposal_account.yes_votes += 1,
            VoteType::No => proposal_account.no_votes += 1,
            VoteType::Abstain => proposal_account.abstain_votes += 1,
        }
        Ok(())
    }

    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
    ) -> Result<()> {
        let proposal_account = &mut ctx.accounts.proposal_account;
        let dao_account = &mut ctx.accounts.dao_account;

        for change in proposal_account.proposed_changes.iter() {
            match change.change_type {
                ChangeType::UpdateMember => {
                    update_member(&change.target, &change.data, ctx.program_id)?;
                }
                ChangeType::UpdateDAO => {
                    update_dao_account(&change.target, &change.data, ctx.program_id)?;
                }
                ChangeType::Other => {
                    // Execute other types of changes
                }
            }
        }

        proposal_account.executed = true;
        Ok(())
    }

    pub fn create_dao(
        ctx: Context<CreateDAO>,
        dao: DAO,
    ) -> Result<()> {
        let dao_account = &mut ctx.accounts.dao_account;
        *dao_account = dao;
        Ok(())
    }

    pub fn update_dao(
        ctx: Context<UpdateDAO>,
        dao_update: DAOUpdate,
    ) -> Result<()> {
        let dao_account = &mut ctx.accounts.dao_account;

        if let Some(new_name) = dao_update.new_name {
            dao_account.name = new_name;
        }
        if let Some(new_description) = dao_update.new_description {
            dao_account.description = new_description;
        }
        if let Some(new_members) = dao_update.new_members {
            dao_account.members = new_members;
        }
        if let Some(new_voting_thresholds) = dao_update.new_voting_thresholds {
            dao_account.voting_thresholds = new_voting_thresholds;
        }

        Ok(())
    }

    pub fn add_member(
        ctx: Context<AddMember>,
        member: Member,
    ) -> Result<()> {
        let dao_account = &mut ctx.accounts.dao_account;
        dao_account.members.push(member.member_pubkey);
        Ok(())
    }

    pub fn remove_member(
        ctx: Context<RemoveMember>,
        member: Member,
    ) -> Result<()> {
        let dao_account = &mut ctx.accounts.dao_account;
        if let Some(index) = dao_account
            .members
            .iter()
            .position(|&m| m == member.member_pubkey)
        {
            dao_account.members.remove(index);
        }
        Ok(())
    }

    pub fn change_voting_thresholds(
        ctx: Context<ChangeVotingThresholds>,
        thresholds: VotingThresholds,
    ) -> Result<()> {
        let dao_account = &mut ctx.accounts.dao_account;
        dao_account.voting_thresholds = thresholds;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposal_account: Account<'info, Proposal>,
    pub dao_account: Account<'info, DAO>,
    pub proposer_account: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal_account: Account<'info, Proposal>,
    pub dao_account: Account<'info, DAO>,
    pub voter_account: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(mut)]
    pub proposal_account: Account<'info, Proposal>,
    #[account(mut)]
    pub dao_account: Account<'info, DAO>,
}

#[derive(Accounts)]
pub struct CreateDAO<'info> {
    #[account(init, payer = creator_account, space = 8 + std::mem::size_of::<DAO>())]
    pub dao_account: Account<'info, DAO>,
    pub creator_account: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateDAO<'info> {
    #[account(mut)]
    pub dao_account: Account<'info, DAO>,
    pub updater_account: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut)]
    pub dao_account: Account<'info, DAO>,
    pub adder_account: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveMember<'info> {
    #[account(mut)]
    pub dao_account: Account<'info, DAO>,
    pub remover_account: Signer<'info>,
}

#[derive(Accounts)]
pub struct ChangeVotingThresholds<'info> {
    #[account(mut)]
    pub dao_account: Account<'info, DAO>,
    pub updater_account: Signer<'info>,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposed_changes: Vec<ProposedChange>,
    pub proposer: Pubkey,
    pub start_time: u64,
    pub end_time: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub abstain_votes: u64,
    pub executed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct ProposedChange {
    pub change_type: ChangeType,
    pub target: Pubkey,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum ChangeType {
    UpdateMember,
    UpdateDAO,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct Vote {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_type: VoteType,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub enum VoteType {
    Yes,
    No,
    Abstain,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct DAO {
    pub id: Pubkey,
    pub name: String,
    pub description: String,
    pub members: Vec<Pubkey>,
    pub voting_thresholds: VotingThresholds,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct DAOUpdate {
    pub dao_id: Pubkey,
    pub new_name: Option<String>,
    pub new_description: Option<String>,
    pub new_members: Option<Vec<Pubkey>>,
    pub new_voting_thresholds: Option<VotingThresholds>,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct Member {
    pub dao_id: Pubkey,
    pub member_pubkey: Pubkey,
}

#[derive(Debug, Clone, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct VotingThresholds {
    pub proposal_creation_threshold: u64,
    pub vote_approval_threshold: u64,
    pub vote_participation_threshold: u64,
}

fn update_member(
    member_pubkey: &Pubkey,
    data: &[u8],
    program_id: &Pubkey,
) -> Result<()> {
    // Implement member update logic here
    Ok(())
}

fn update_dao_account(
    dao_pubkey: &Pubkey,
    data: &[u8],
    program_id: &Pubkey,
) -> Result<()> {
    // Implement DAO update logic here
    Ok(())
}