// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import {IERC20, IERC20Metadata} from "@/libraries/interfaces/IERC20.sol";
import {IPool} from "@/libraries/interfaces/IPool.sol";
import {IUniswapV3Router} from "@/libraries/interfaces/IUniswapV3Router.sol";
import {ICurvePool} from "@/libraries/interfaces/ICurvePool.sol";
import {IBalancerVault} from "@/libraries/interfaces/IBalancerVault.sol";
import {CallbackValidation} from "@/libraries/CallbackValidation.sol";

/**
 * @title MockExecutor
 * @notice Smart contract used for simulating transaction effects without execution
 * Used in off-chain simulation phase (Phase 2-4) to validate profit potential
 */
contract MockExecutor {
    address public immutable owner;
    mapping(bytes32 => bool) public simulationExecuted;

    error NotOwner();
    error SimulationAlreadyExecuted(bytes32 hash);
    error SimulationFailed(bytes32 hash, bytes reason);
    error InvalidCalldata();

    event SimulationExecuted(bytes32 indexed hash, address target, uint256 profit, uint256 gasUsed);
    event SimulationFailed(bytes32 indexed hash, address target, bytes reason);

    constructor() {
        owner = msg.sender;
    }

    modifier onlyOwner() {
        if (msg.sender != owner) revert NotOwner();
        _;
    }

    /**
     * @notice Simulates execution of arbitrary call data against this contract
     * @dev This is the core simulation mechanism - all strategies' transactions
     *      must be formatted to call this function via the Ethereum node
     * @param data Raw calldata representing the transaction to simulate
     * @return success Whether the simulation succeeded
     * @return result Hex-encoded transaction trace with detailed state changes
     */
    function simulate(bytes calldata data) external payable onlyOwner returns (bool success, bytes memory result) {
        bytes32 simHash = keccak256(data);
        if (simulationExecuted[simHash]) revert SimulationAlreadyExecuted(simHash);
        simulationExecuted[simHash] = true;

        (success, result) = address(this).call{value: msg.value}(data);

        if (success) {
            emit SimulationExecuted(simHash, msg.sender, extractProfit(result), extractGasUsed(result));
        } else {
            emit SimulationFailed(simHash, msg.sender, extractReason(result));
        }

        return (success, result);
    }

    /**
     * @notice Helper function to extract profit from simulation trace
     * @dev Analyzes ERC20 balance changes to calculate net profit
     *      Assumes call originates from Aave flash loan repayment context
     * @param trace Transaction trace data from simulation
     * @return profit Amount of profit calculated (in USD, based on token balances)
     */
    function extractProfit(bytes memory trace) internal pure returns (uint256 profit) {
        (bool success, bytes memory data) = abi.decode(trace, (bool, bytes));
        if (!success) return 0;

        (uint256 beforeAssets, uint256 afterAssets) = abi.decode(data, (uint256, uint256));
        return afterAssets > beforeAssets ? afterAssets - beforeAssets : 0;
    }

    /**
     * @notice Extracts gas used from simulation result
     * @param trace Transaction trace data from simulation
     * @return gasUsed Gas consumed during transaction execution
     */
    function extractGasUsed(bytes memory trace) internal pure returns (uint256 gasUsed) {
        (bool success, bytes memory data) = abi.decode(trace, (bool, bytes));
        if (!success) return 0;
        return abi.decode(data, (uint256));
    }

    /**
     * @notice Extract revert reason from failed simulation
     * @dev Converts return data to human-readable error message
     * @param data Encoded revert reason data
     * @return reason Human-readable error description
     */
    function extractReason(bytes memory data) internal pure returns (bytes memory reason) {
        if (data.length == 0) return "Unknown error";
        return data;
    }
}