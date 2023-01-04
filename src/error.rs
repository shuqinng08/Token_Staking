use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("You used wrong token contract")]
    WrongContractError {},

    #[error("You should send the bond message when you call this function")]
    DataShouldBeGiven {},

    #[error("Cannot update; the new schedule must support all of the previous schedule")]
    NotIncludeAllDistributionSchedule {},

    #[error("new schedule removes already started distribution")]
    NewScheduleRemovePastDistribution {},

    #[error("new schedule adds an already started distribution")]
    NewScheduleAddPastDistribution {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },
}
