// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.9;

import {ECDSAServiceManagerBase} from
    "@eigenlayer-middleware/src/unaudited/ECDSAServiceManagerBase.sol";
import {ECDSAStakeRegistry} from "@eigenlayer-middleware/src/unaudited/ECDSAStakeRegistry.sol";
import {IServiceManager} from "@eigenlayer-middleware/src/interfaces/IServiceManager.sol";
import {ECDSAUpgradeable} from
    "@openzeppelin-upgrades/contracts/utils/cryptography/ECDSAUpgradeable.sol";
import {IERC1271Upgradeable} from "@openzeppelin-upgrades/contracts/interfaces/IERC1271Upgradeable.sol";
import {IHelloWorldServiceManager} from "./IHelloWorldServiceManager.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "@eigenlayer/contracts/interfaces/IRewardsCoordinator.sol";
//import {TransparentUpgradeableProxy} from "@openzeppelin/contracts/proxy/transparent/TransparentUpgradeableProxy.sol";

contract HelloWorldServiceManager is ECDSAServiceManagerBase, IHelloWorldServiceManager {
    using ECDSAUpgradeable for bytes32;

    uint32 public latestTaskNum;

    // Маппинги для базовой функциональности
    mapping(uint32 => bytes32) public allTaskHashes;
    mapping(address => mapping(uint32 => bytes)) public allTaskResponses;

    // Структура для Pastebin задачи
    struct PasteContent {
        uint256 id;
        address creator;
        string content;
        uint256 timestamp;
    }

    // Хранение паст
    mapping(uint256 => PasteContent) public pastes;
    uint256 public pasteCount;

    // События
    event PasteCreated(uint256 indexed id, address indexed creator, string content, uint256 timestamp);

    constructor(
        address _avsDirectory,
        address _stakeRegistry,
        address _rewardsCoordinator,
        address _delegationManager
    )
        ECDSAServiceManagerBase(
            _avsDirectory,
            _stakeRegistry,
            _rewardsCoordinator,
            _delegationManager
        )
    {}

    // Реализация методов из интерфейса IHelloWorldServiceManager
    function createNewTask(
        string memory name
    ) external returns (Task memory) {
        Task memory newTask;
        newTask.name = name;
        newTask.taskCreatedBlock = uint32(block.number);

        allTaskHashes[latestTaskNum] = keccak256(abi.encode(newTask));
        emit NewTaskCreated(latestTaskNum, newTask);
        latestTaskNum = latestTaskNum + 1;

        return newTask;
    }

    function respondToTask(
        Task calldata task,
        uint32 referenceTaskIndex,
        bytes memory signature
    ) external onlyOperator {
        require(
            keccak256(abi.encode(task)) == allTaskHashes[referenceTaskIndex],
            "Invalid task data"
        );
        require(
            allTaskResponses[msg.sender][referenceTaskIndex].length == 0,
            "Operator already responded"
        );

        bytes32 messageHash = keccak256(abi.encodePacked("Hello, ", task.name));
        bytes32 ethSignedMessageHash = messageHash.toEthSignedMessageHash();
        
        bytes4 magicValue = IERC1271Upgradeable.isValidSignature.selector;
        require(
            magicValue == ECDSAStakeRegistry(stakeRegistry).isValidSignature(ethSignedMessageHash, signature),
            "Invalid signature"
        );

        allTaskResponses[msg.sender][referenceTaskIndex] = signature;
        emit TaskResponded(referenceTaskIndex, task, msg.sender);
    }

    // Функции для работы с пастами
    function createPaste(string calldata content) external returns (uint256) {
        uint256 pasteId = pasteCount;
        
        pastes[pasteId] = PasteContent({
            id: pasteId,
            creator: msg.sender,
            content: content,
            timestamp: block.timestamp
        });
        
        emit PasteCreated(pasteId, msg.sender, content, block.timestamp);
        
        pasteCount++;
        return pasteId;
    }

    function getPaste(uint256 id) external view returns (PasteContent memory) {
        require(id < pasteCount, "Paste does not exist");
        return pastes[id];
    }

    modifier onlyOperator() {
        require(
            ECDSAStakeRegistry(stakeRegistry).operatorRegistered(msg.sender),
            "Operator must be the caller"
        );
        _;
    }
}
