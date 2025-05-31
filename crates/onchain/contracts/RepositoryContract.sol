// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {AccessControl} from "@openzeppelin/contracts/access/AccessControl.sol";

contract RepositoryContract is AccessControl {

        bytes32 public constant PUSHER_ROLE = keccak256("PUSHER_ROLE");


    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(PUSHER_ROLE, msg.sender);
    }

    struct Object {
        string hash;
        bytes ipfs_url;
        address pusher;
    }

    struct Ref {
        string name;
        bytes data;
        bool is_active;
        address pusher;
    }

    mapping(string => Object) public objects;
    mapping(string => Ref) public refs;
    mapping(string => uint256) private refIndex;

    Object[] public objectsById;
    Ref[] public refsById;

    bytes public config;

    event ObjectSaved(string hash, bytes ipfs_url, address pusher);
    event RefAdded(string ref, bytes ipfs_url, address pusher);
    event ConfigUpdated(bytes config);

    modifier onlyPusher() {
        require(hasRole(PUSHER_ROLE, msg.sender), "Caller is not a pusher");
        _;
    }

    modifier onlyAdmin() {
        require(hasRole(DEFAULT_ADMIN_ROLE, msg.sender), "Caller is not an admin");
        _;
    }

    function saveObject(string memory _hash, bytes memory _ipfs_url) public onlyPusher {
        if (objects[_hash].ipfs_url.length > 0) {
            return;
        }

        Object memory object = Object(_hash, _ipfs_url, msg.sender);
        objects[_hash] = object;
        objectsById.push(object);
        emit ObjectSaved(_hash, _ipfs_url, msg.sender);
    }

    function addRef(string memory _ref, bytes memory _data) public onlyPusher {
        address pusher = msg.sender;
        Ref memory newRef = Ref(_ref, _data, true, pusher);

        if (refs[_ref].is_active) {
            uint256 idx = refIndex[_ref];
            refsById[idx] = newRef;
            refs[_ref] = newRef;
        } else {
            refIndex[_ref] = refsById.length;
            refsById.push(newRef);
            refs[_ref] = newRef;
        }
        emit RefAdded(_ref, _data, pusher);
    }

    function updateConfig(bytes memory _config) public onlyPusher {
        config = _config;
        emit ConfigUpdated(_config);
    }

    function grantPusherRole(address _address) public onlyAdmin {
        _grantRole(PUSHER_ROLE, _address);
    }

    function revokePusherRole(address _address) public onlyAdmin {
        _revokeRole(PUSHER_ROLE, _address);
    }

    function hasPusherRole(address _address) public view returns (bool) {
        return hasRole(PUSHER_ROLE, _address);
    }

    function grantAdminRole(address _address) public onlyAdmin {
        _grantRole(DEFAULT_ADMIN_ROLE, _address);
    }

    function revokeAdminRole(address _address) public onlyAdmin {
        _revokeRole(DEFAULT_ADMIN_ROLE, _address);
    }

    function hasAdminRole(address _address) public view returns (bool) {
        return hasRole(DEFAULT_ADMIN_ROLE, _address);
    }

    function getConfig() public view returns (bytes memory) {
        return config;
    }

    function getObjectById(uint256 _id) public view returns (Object memory) {
        return objectsById[_id];
    }

    function getObject(string memory _hash) public view returns (Object memory) {
        return objects[_hash];
    }

    function isObjectExist(string memory _hash) public view returns (bool) {
        return objects[_hash].ipfs_url.length > 0;
    }

    function checkObjects(string[] memory _hashes) public view returns (bool[] memory) {
        bool[] memory results = new bool[](_hashes.length);
        for (uint256 i = 0; i < _hashes.length; i++) {
            results[i] = objects[_hashes[i]].ipfs_url.length > 0;
        }
        return results;
    }

    function addObjects(string[] memory _hashes, bytes[] memory _ipfs_urls) public onlyPusher {
        for (uint256 i = 0; i < _hashes.length; i++) {
            if (objects[_hashes[i]].ipfs_url.length > 0) {
                continue;
            }
            Object memory object = Object(_hashes[i], _ipfs_urls[i], msg.sender);
            objects[_hashes[i]] = object;
            objectsById.push(object);
            emit ObjectSaved(_hashes[i], _ipfs_urls[i], msg.sender);
        }
    }

    function addRefs(string[] memory _refsArr, bytes[] memory _dataArr) public onlyPusher {
        require(_refsArr.length == _dataArr.length, "Mismatched refs and data arrays");
        address pusher = msg.sender;
        for (uint256 i = 0; i < _refsArr.length; i++) {
            string memory name = _refsArr[i];
            bytes memory data = _dataArr[i];
            Ref memory newRef = Ref(name, data, true, pusher);


            if (refs[name].is_active) {
                uint256 idx = refIndex[name];
                refsById[idx] = newRef;
                refs[name] = newRef;
            } else {
                refIndex[name] = refsById.length;
                refsById.push(newRef);
                refs[name] = newRef;
            }
            emit RefAdded(name, data, pusher);
        }
    }

    function getRefs() public view returns (Ref[] memory) {
        return refsById;
    }

    function getObjects() public view returns (Object[] memory) {
        return objectsById;
    }

    function getObjectsLength() public view returns (uint256) {
        return objectsById.length;
    }

    function getRefsLength() public view returns (uint256) {
        return refsById.length;
    }

    function getRefById(uint256 _id) public view returns (Ref memory) {
        return refsById[_id];
    }
}
