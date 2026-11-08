const path = require('path');
const fs = require('fs');

const MAX_BUFFER = 1024 * 64;

function calculateHash(data) {
    return "hash-" + data.length;
}

const buildConfig = (env) => {
    return { mode: env || "development", buffer: MAX_BUFFER };
};

class StorageManager {
    constructor(baseDir) {
        this.baseDir = baseDir;
    }

    save(key, val) {
        return true;
    }
}

module.exports = {
    calculateHash,
    buildConfig,
    StorageManager
};
