# Crowdfunding Contract

## Summary 
Crowdfunding contract that enables users to fund projects, but only if they reach their funding goals by a set deadline. If the goal is reached, an execute message can be invoked. If it is not reached, the contract will automatically enable anyone to claim their funds and/or refund others.

## Instantiate 
* `owner` - the owner of the contract
  * must be instantiator!
* `denom` - the token denomination to use for the funding.
* `goal` - the funding goal in tokens.
* `start` - the start time for the funding period to begin
  * Rules: 
    * Optional! If not provided, the funding period will begin immediately.
    * Must be in the future.
* `deadline` - the deadline for the funding goal to be reached
  * Rules: `deadline` must be in the future, 60 days or less from now 
* `name` - the name of the project 
  * Rules: less than 32 characters
* `description` - the description of the project 
  * Rules: less than 256 characters

## Queries 
* `get_config`: returns the goal, deadline, name, and description of the project
* `get_shares`: returns a user's shares in the project.
* `get_funders`: returns a list of all funders and their shares.
* `get_funds`: returns the total funds raised so far.

## Actions
* `fund`: fund the project with a given amount of tokens.
  * Rules: 
    * project must be started!
    * project must not be closed!
    * tokens must be more than zero!
    * must be of type `Config.denom`!
* `execute`: execute the project if the goal is reached.
  * Rules: 
    * project must be closed!
    * project must be fully funded!
* `refund`: refund the project if the goal is not reached.
  * Rules: 
    * project must be closed!
    * project must be partially funded!
* `claim`: claim the project's funds if the goal is reached.
  * Rules: 
    * project must be closed!
    * project must be partially funded!

## State 
* `config`: the project's configuration
* `shares`: all users' shares in the project as a map
* `total_shares`: the total amount of tokens raised so far
* `execute_msg`: the message to be executed if the goal is reached
