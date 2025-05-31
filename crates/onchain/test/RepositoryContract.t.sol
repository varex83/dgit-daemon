// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {RepositoryContract} from "../contracts/RepositoryContract.sol";

contract RepositoryContractTest is Test {
    RepositoryContract public repositoryContract;

    // Test addresses
    address public admin;
    address public pusher1;
    address public pusher2;
    address public unauthorized;

    // Test data
    string constant HASH1 = "abc123def456";
    string constant HASH2 = "def456ghi789";
    string constant HASH3 = "ghi789jkl012";
    bytes constant IPFS_URL1 = "QmTest1234567890";
    bytes constant IPFS_URL2 = "QmTest0987654321";
    bytes constant IPFS_URL3 = "QmTest1122334455";

    string constant REF1 = "refs/heads/main";
    string constant REF2 = "refs/heads/develop";
    string constant REF3 = "refs/tags/v1.0.0";
    bytes constant REF_DATA1 = "commit_hash_main";
    bytes constant REF_DATA2 = "commit_hash_develop";
    bytes constant REF_DATA3 = "commit_hash_tag";

    bytes constant CONFIG_DATA = "repository_config_data";

    function setUp() public {
        admin = address(this); // Test contract is admin by default
        pusher1 = makeAddr("pusher1");
        pusher2 = makeAddr("pusher2");
        unauthorized = makeAddr("unauthorized");

        repositoryContract = new RepositoryContract();

        // Grant pusher roles to test addresses
        repositoryContract.grantPusherRole(pusher1);
        repositoryContract.grantPusherRole(pusher2);
    }

    // ============ Role Management Tests ============

    function test_grantPusherRole() public {
        repositoryContract.grantPusherRole(address(this));
        assertEq(repositoryContract.hasPusherRole(address(this)), true);
    }

    function test_revokePusherRole() public {
        repositoryContract.grantPusherRole(address(this));
        repositoryContract.revokePusherRole(address(this));
        assertEq(repositoryContract.hasPusherRole(address(this)), false);
    }

    function test_grantAdminRole() public {
        address newAdmin = makeAddr("newAdmin");
        repositoryContract.grantAdminRole(newAdmin);
        assertEq(repositoryContract.hasAdminRole(newAdmin), true);
    }

    function test_revokeAdminRole() public {
        address newAdmin = makeAddr("newAdmin");
        repositoryContract.grantAdminRole(newAdmin);
        repositoryContract.revokeAdminRole(newAdmin);
        assertEq(repositoryContract.hasAdminRole(newAdmin), false);
    }

    function test_onlyAdminCanGrantRoles() public {
        vm.prank(unauthorized);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.grantPusherRole(unauthorized);

        vm.prank(unauthorized);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.grantAdminRole(unauthorized);
    }

    function test_onlyAdminCanRevokeRoles() public {
        vm.prank(unauthorized);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.revokePusherRole(pusher1);

        vm.prank(unauthorized);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.revokeAdminRole(admin);
    }

    // ============ Object Management Tests ============

    function test_saveObject() public {
        vm.prank(pusher1);
        vm.expectEmit(true, true, true, true);
        emit RepositoryContract.ObjectSaved(HASH1, IPFS_URL1, pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);

        RepositoryContract.Object memory obj = repositoryContract.getObject(HASH1);
        assertEq(obj.hash, HASH1);
        assertEq(obj.ipfs_url, IPFS_URL1);
        assertEq(obj.pusher, pusher1);
        assertEq(repositoryContract.isObjectExist(HASH1), true);
    }

    function test_saveObjectOnlyPusher() public {
        vm.prank(unauthorized);
        vm.expectRevert("Caller is not a pusher");
        repositoryContract.saveObject(HASH1, IPFS_URL1);
    }

    function test_saveObjectIdempotent() public {
        vm.prank(pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);

        uint256 initialLength = repositoryContract.getObjectsLength();

        // Saving the same object again should not create duplicate
        vm.prank(pusher2);
        repositoryContract.saveObject(HASH1, IPFS_URL2); // Different IPFS URL

        assertEq(repositoryContract.getObjectsLength(), initialLength);

        // Original object should remain unchanged
        RepositoryContract.Object memory obj = repositoryContract.getObject(HASH1);
        assertEq(obj.ipfs_url, IPFS_URL1);
        assertEq(obj.pusher, pusher1);
    }

    function test_addObjects() public {
        string[] memory hashes = new string[](3);
        bytes[] memory ipfsUrls = new bytes[](3);

        hashes[0] = HASH1;
        hashes[1] = HASH2;
        hashes[2] = HASH3;
        ipfsUrls[0] = IPFS_URL1;
        ipfsUrls[1] = IPFS_URL2;
        ipfsUrls[2] = IPFS_URL3;

        vm.prank(pusher1);
        repositoryContract.addObjects(hashes, ipfsUrls);

        assertEq(repositoryContract.getObjectsLength(), 3);
        assertEq(repositoryContract.isObjectExist(HASH1), true);
        assertEq(repositoryContract.isObjectExist(HASH2), true);
        assertEq(repositoryContract.isObjectExist(HASH3), true);

        RepositoryContract.Object memory obj1 = repositoryContract.getObjectById(0);
        assertEq(obj1.hash, HASH1);
        assertEq(obj1.ipfs_url, IPFS_URL1);
        assertEq(obj1.pusher, pusher1);
    }

    function test_addObjectsOnlyPusher() public {
        string[] memory hashes = new string[](1);
        bytes[] memory ipfsUrls = new bytes[](1);
        hashes[0] = HASH1;
        ipfsUrls[0] = IPFS_URL1;

        vm.prank(unauthorized);
        vm.expectRevert("Caller is not a pusher");
        repositoryContract.addObjects(hashes, ipfsUrls);
    }

    function test_checkObjects() public {
        vm.prank(pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);
        vm.prank(pusher1);
        repositoryContract.saveObject(HASH3, IPFS_URL3);

        string[] memory hashesToCheck = new string[](3);
        hashesToCheck[0] = HASH1;
        hashesToCheck[1] = HASH2; // Not saved
        hashesToCheck[2] = HASH3;

        bool[] memory results = repositoryContract.checkObjects(hashesToCheck);
        assertEq(results[0], true);
        assertEq(results[1], false);
        assertEq(results[2], true);
    }

    function test_getObjects() public {
        vm.prank(pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);
        vm.prank(pusher2);
        repositoryContract.saveObject(HASH2, IPFS_URL2);

        RepositoryContract.Object[] memory objects = repositoryContract.getObjects();
        assertEq(objects.length, 2);
        assertEq(objects[0].hash, HASH1);
        assertEq(objects[1].hash, HASH2);
    }

    // ============ Ref Management Tests ============

    function test_addRef() public {
        vm.prank(pusher1);
        vm.expectEmit(true, true, true, true);
        emit RepositoryContract.RefAdded(REF1, REF_DATA1, pusher1);
        repositoryContract.addRef(REF1, REF_DATA1);

        (string memory name, bytes memory data, bool is_active, address pusher) = repositoryContract.refs(REF1);
        assertEq(name, REF1);
        assertEq(data, REF_DATA1);
        assertEq(is_active, true);
        assertEq(pusher, pusher1);
        assertEq(repositoryContract.getRefsLength(), 1);
    }

    function test_addRefOnlyPusher() public {
        vm.prank(unauthorized);
        vm.expectRevert("Caller is not a pusher");
        repositoryContract.addRef(REF1, REF_DATA1);
    }

    function test_updateExistingRef() public {
        vm.prank(pusher1);
        repositoryContract.addRef(REF1, REF_DATA1);

        uint256 initialLength = repositoryContract.getRefsLength();

        // Update the same ref with new data
        vm.prank(pusher2);
        repositoryContract.addRef(REF1, REF_DATA2);

        // Length should remain the same (updated, not added)
        assertEq(repositoryContract.getRefsLength(), initialLength);

        (, bytes memory data, , address pusher) = repositoryContract.refs(REF1);
        assertEq(data, REF_DATA2);
        assertEq(pusher, pusher2);
    }

    function test_addRefs() public {
        string[] memory refs = new string[](3);
        bytes[] memory data = new bytes[](3);

        refs[0] = REF1;
        refs[1] = REF2;
        refs[2] = REF3;
        data[0] = REF_DATA1;
        data[1] = REF_DATA2;
        data[2] = REF_DATA3;

        vm.prank(pusher1);
        repositoryContract.addRefs(refs, data);

        assertEq(repositoryContract.getRefsLength(), 3);

        RepositoryContract.Ref memory ref1 = repositoryContract.getRefById(0);
        assertEq(ref1.name, REF1);
        assertEq(ref1.data, REF_DATA1);
        assertEq(ref1.pusher, pusher1);
    }

    function test_addRefsOnlyPusher() public {
        string[] memory refs = new string[](1);
        bytes[] memory data = new bytes[](1);
        refs[0] = REF1;
        data[0] = REF_DATA1;

        vm.prank(unauthorized);
        vm.expectRevert("Caller is not a pusher");
        repositoryContract.addRefs(refs, data);
    }

    function test_addRefsMismatchedArrays() public {
        string[] memory refs = new string[](2);
        bytes[] memory data = new bytes[](1);
        refs[0] = REF1;
        refs[1] = REF2;
        data[0] = REF_DATA1;

        vm.prank(pusher1);
        vm.expectRevert("Mismatched refs and data arrays");
        repositoryContract.addRefs(refs, data);
    }

    function test_getRefs() public {
        vm.prank(pusher1);
        repositoryContract.addRef(REF1, REF_DATA1);
        vm.prank(pusher1);
        repositoryContract.addRef(REF2, REF_DATA2);

        RepositoryContract.Ref[] memory refs = repositoryContract.getRefs();
        assertEq(refs.length, 2);
        assertEq(refs[0].name, REF1);
        assertEq(refs[1].name, REF2);
    }

    // ============ Config Management Tests ============

    function test_updateConfig() public {
        vm.prank(pusher1);
        vm.expectEmit(true, false, false, true);
        emit RepositoryContract.ConfigUpdated(CONFIG_DATA);
        repositoryContract.updateConfig(CONFIG_DATA);

        bytes memory config = repositoryContract.getConfig();
        assertEq(config, CONFIG_DATA);
    }

    function test_updateConfigOnlyPusher() public {
        vm.prank(unauthorized);
        vm.expectRevert("Caller is not a pusher");
        repositoryContract.updateConfig(CONFIG_DATA);
    }

    function test_getConfigInitiallyEmpty() public view {
        bytes memory config = repositoryContract.getConfig();
        assertEq(config.length, 0);
    }

    // ============ Edge Cases and Complex Scenarios ============

    function test_mixedOperationsWithDifferentPushers() public {
        // pusher1 adds objects and refs
        vm.prank(pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);
        vm.prank(pusher1);
        repositoryContract.addRef(REF1, REF_DATA1);

        // pusher2 adds different objects and updates ref
        vm.prank(pusher2);
        repositoryContract.saveObject(HASH2, IPFS_URL2);
        vm.prank(pusher2);
        repositoryContract.addRef(REF1, REF_DATA2); // Update existing ref

        // Verify objects are from different pushers
        RepositoryContract.Object memory obj1 = repositoryContract.getObject(HASH1);
        RepositoryContract.Object memory obj2 = repositoryContract.getObject(HASH2);
        assertEq(obj1.pusher, pusher1);
        assertEq(obj2.pusher, pusher2);

        // Verify ref was updated by pusher2
        (, bytes memory refData, , address refPusher) = repositoryContract.refs(REF1);
        assertEq(refPusher, pusher2);
        assertEq(refData, REF_DATA2);
    }

    function test_emptyArrayOperations() public {
        string[] memory emptyHashes = new string[](0);
        bytes[] memory emptyIpfsUrls = new bytes[](0);
        string[] memory emptyRefs = new string[](0);
        bytes[] memory emptyData = new bytes[](0);

        vm.prank(pusher1);
        repositoryContract.addObjects(emptyHashes, emptyIpfsUrls);
        vm.prank(pusher1);
        repositoryContract.addRefs(emptyRefs, emptyData);

        assertEq(repositoryContract.getObjectsLength(), 0);
        assertEq(repositoryContract.getRefsLength(), 0);
    }

    function test_largeBatchOperations() public {
        uint256 batchSize = 10;
        string[] memory hashes = new string[](batchSize);
        bytes[] memory ipfsUrls = new bytes[](batchSize);
        string[] memory refs = new string[](batchSize);
        bytes[] memory data = new bytes[](batchSize);

        for (uint256 i = 0; i < batchSize; i++) {
            hashes[i] = string(abi.encodePacked("hash", vm.toString(i)));
            ipfsUrls[i] = abi.encodePacked("ipfs", i);
            refs[i] = string(abi.encodePacked("refs/heads/branch", vm.toString(i)));
            data[i] = abi.encodePacked("commit", i);
        }

        vm.prank(pusher1);
        repositoryContract.addObjects(hashes, ipfsUrls);
        vm.prank(pusher1);
        repositoryContract.addRefs(refs, data);

        assertEq(repositoryContract.getObjectsLength(), batchSize);
        assertEq(repositoryContract.getRefsLength(), batchSize);

        // Verify first and last items
        RepositoryContract.Object memory firstObj = repositoryContract.getObjectById(0);
        RepositoryContract.Object memory lastObj = repositoryContract.getObjectById(batchSize - 1);
        assertEq(firstObj.hash, "hash0");
        assertEq(lastObj.hash, string(abi.encodePacked("hash", vm.toString(batchSize - 1))));
    }

    // ============ Access Control Edge Cases ============

    function test_adminCannotCallPusherFunctions() public {
        // Admin role doesn't include pusher permissions by default
        // But constructor grants both roles to deployer, so we need to revoke pusher role first
        repositoryContract.revokePusherRole(admin);

        vm.expectRevert("Caller is not a pusher");
        repositoryContract.saveObject(HASH1, IPFS_URL1);

        vm.expectRevert("Caller is not a pusher");
        repositoryContract.addRef(REF1, REF_DATA1);

        vm.expectRevert("Caller is not a pusher");
        repositoryContract.updateConfig(CONFIG_DATA);
    }

    function test_pusherCannotGrantRoles() public {
        vm.prank(pusher1);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.grantPusherRole(unauthorized);

        vm.prank(pusher1);
        vm.expectRevert("Caller is not an admin");
        repositoryContract.grantAdminRole(unauthorized);
    }

    // ============ View Functions Tests ============

    function test_viewFunctionsWithoutData() public view {
        assertEq(repositoryContract.getObjectsLength(), 0);
        assertEq(repositoryContract.getRefsLength(), 0);
        assertEq(repositoryContract.isObjectExist("nonexistent"), false);

        RepositoryContract.Object[] memory objects = repositoryContract.getObjects();
        RepositoryContract.Ref[] memory refs = repositoryContract.getRefs();
        assertEq(objects.length, 0);
        assertEq(refs.length, 0);
    }

    function test_getObjectByIdOutOfBounds() public {
        vm.expectRevert();
        repositoryContract.getObjectById(0);
    }

    function test_getRefByIdOutOfBounds() public {
        vm.expectRevert();
        repositoryContract.getRefById(0);
    }

    // ============ Event Testing ============

    function test_objectSavedEventEmission() public {
        vm.prank(pusher1);
        vm.expectEmit(true, true, true, true);
        emit RepositoryContract.ObjectSaved(HASH1, IPFS_URL1, pusher1);
        repositoryContract.saveObject(HASH1, IPFS_URL1);
    }

    function test_refAddedEventEmission() public {
        vm.prank(pusher1);
        vm.expectEmit(true, true, true, true);
        emit RepositoryContract.RefAdded(REF1, REF_DATA1, pusher1);
        repositoryContract.addRef(REF1, REF_DATA1);
    }

    function test_configUpdatedEventEmission() public {
        vm.prank(pusher1);
        vm.expectEmit(true, false, false, true);
        emit RepositoryContract.ConfigUpdated(CONFIG_DATA);
        repositoryContract.updateConfig(CONFIG_DATA);
    }
}
