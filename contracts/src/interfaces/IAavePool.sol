// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

interface IAavePool {
    function supply(address asset, uint256 amount, address onBehalfOf, uint16 referralCode) external;
    function borrow(address asset, uint256 amount, uint256 interestRateMode, uint16 referralCode, address onBehalfOf) external;
    function repay(address asset, uint256 amount, uint256 interestRateMode, address onBehalfOf) external returns (uint256);
    function withdraw(address asset, uint256 amount, address to) external returns (uint256);
    function flashLoan(address receiverAddress, address[] memory assets, uint256[] memory amounts, uint256[] memory modes, address onBehalfOf, bytes memory params) external;
    function getUserAccountData(address user) external view returns (
        uint256 totalCollateralETH,
        uint256 totalDebtETH,
        uint256 availableBorrowsETH,
        uint256 currentLiquidationThreshold,
        uint256 ltv,
        uint256 healthFactor
    );
}

interface IPool {
    function flashLoanSimple(address receiver, address token, uint256 amount, bytes calldata data, uint16 referralCode) external;
    function flashLoan(address receiverAddress, address[] memory assets, uint256[] memory amounts, uint256[] memory interestRateModes, address onBehalfOf, bytes memory data, uint16 referralCode) external;
}