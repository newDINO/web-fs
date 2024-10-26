"use strict";

// Changes to this file is automatically copied to worker_static.js using build script.
// The build script runs before running wasm-bindgen, so changes will to this file will always take effect.

/**
 * @class
 * @constructor
 * @public
 * @template T
 */
class Arena {
    constructor() {
        /**
         * @type {(T | null)[]}
         * @private
         */
        this.inner = [];
        /**
         * @type {number | null}
         * @private
         */
        this.lastRemove = null;
    }
    /**
     * @param {T} e 
     * @returns {number}
     */
    insert(e) {
        if(this.lastRemove != null) {
            let vacancy = this.lastRemove;
            this.inner[vacancy] = e;
            this.lastRemove = null;
            return vacancy;
        } else {
            for(let i = 0; i < this.inner.length; i++) {
                if(this.inner[i] == null) {
                    this.inner[i] = e;
                    return i;
                }
            }
            return this.inner.push(e) - 1;
        }
    }
    /**
     * @param {number} i
     * @returns {T | null} 
     */
    remove(i) {
        if(i >= this.inner.length) {
            return null;
        }
        let result = this.inner[i];
        this.inner[i] = null;
        return result;
    }
    /**
     * @param {number} i
     * @returns {T | null}
     */
    get(i) {
        if(i >= this.inner.length) {
            return null;
        }
        return this.inner[i];
    }
}

const APPEND = 0b0000_0001;
const CREATE = 0b0000_0010;
const CREATE_NEW = 0b0000_0100;
const READ = 0b0000_1000;
const TRUNCATE = 0b0001_0000;
const WRITE = 0b0010_0000;

let opened = new Arena();

onmessage = async (e) => {
    console.log("Message received");
    let msg = e.data;
    console.log(msg);
    if(msg.Open != undefined) {
        /**
         * @typedef InOpenMsg
         * @type {object}
         * @property {number} options
         * @property {FileSystemFileHandle} handle
         * @property {number} index
         */
        /**
         * @type {InOpenMsg}
         */
        let openMsg = msg.Open;

        let openOptions;
        if((openMsg.options & WRITE) > 0) {
            openOptions = {
                mode: "readwrite"
            }
        } else {
            openOptions = {
                mode: "read-only"
            }
        }

        let response = {
            0: {
                index: openMsg.index,
            }
        };
        try {
            let accessHandle = await openMsg.handle.createSyncAccessHandle(openOptions);
            let fd = opened.insert(accessHandle);
            response[0].fd = fd;
        } catch (error) {
            response.error = error.toString() + JSON.stringify(openMsg);
        } finally {
            console.log(response);
            postMessage(response);
        }
    } else if(msg.Drop != undefined) {
        /**
         * @typedef InDropMsg
         * @type {object}
         * @property {number} fd
         */
        /**
         * @type {InDropMsg}
         */
        let dropMsg = msg.Drop;
        let accessHandle = opened.remove(dropMsg.fd);
        accessHandle.close();
    } else if(msg.Read != undefined) {
        /**
         * @typedef InReadMsg
         * @type {object}
         * @property {number} fd
         * @property {number} size
         * @property {number} index
         */
        /**
         * @type {InReadMsg}
         */
        let readMsg = msg.Read;

        let response = {
            1: {
                index: readMsg.index,
            }
        };
        try {
            let accessHandle = opened.get(readMsg.fd);
            let buffer = new ArrayBuffer(readMsg.size);
            let size = accessHandle.read(buffer);
            response[1].buf = buffer;
            response[1].size = size;
        } catch (error) {
            response.error = error.toString();
        } finally {
            console.log(response);
            postMessage(response);
        }
    } else if(msg.Write != undefined) {
        /**
         * @typedef InWriteMsg
         * @type {object}
         * @property {number} fd
         * @property {ArrayBuffer} buf
         * @property {number} index
         */
        /**
         * @type {InWriteMsg}
         */
        let writeMsg = msg.Write;

        let response = {
            2: {
                index: writeMsg.index,
            }
        };
        try {
            let accessHandle = opened.get(writeMsg.fd);
            let size = accessHandle.write(writeMsg.buf);
            response[2].size = size;
        } catch (error) {
            response.error = error.toString();
        } finally {
            console.log(response);
            postMessage(response);
        }
    } else if(msg.Flush != undefined) {
        /**
         * @typedef InFlushMsg
         * @type {object}
         * @property {number} fd
         * @property {number} index
         */
        /**
         * @type {InFlushMsg}
         */
        let flushMsg = msg.Flush;

        let response = {
            3: {
                index: flushMsg.index,
            }
        };
        try {
            let accessHandle = opened.get(flushMsg.fd);
            accessHandle.flush();
        } catch (error) {
            response.error = error.toString();
        } finally {
            console.log(response);
            postMessage(response);
        }
    } else if(msg.Close != undefined) {
        /**
         * @typedef InCloseMsg
         * @type {object}
         * @property {number} fd
         * @property {number} index
         */
        /**
         * @type {InCloseMsg}
         */
        let closeMsg = msg.Close;

        let response = {
            4: {
                index: closeMsg.index
            }
        };
        try {
            let accessHandle = opened.get(closeMsg.fd);
            accessHandle.close();
        } catch (error) {
            response.error = error.toString();
        } finally {
            console.log(response);
            postMessage(response);
        }
    }
}