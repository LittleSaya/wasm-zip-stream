let wasm;

const heap = new Array(128).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_2.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_2.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}
function __wbg_adapter_24(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h5b3f9cd6c38a6389(arg0, arg1, addHeapObject(arg2));
}

function __wbg_adapter_33(arg0, arg1, arg2) {
    wasm._dyn_core__ops__function__FnMut__A____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1c265f1720677f49(arg0, arg1, addHeapObject(arg2));
}

/**
 * @param {Function} create_writer
 * @returns {Handles}
 */
export function initialize_context(create_writer) {
    const ret = wasm.initialize_context(addHeapObject(create_writer));
    return Handles.__wrap(ret);
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_exn_store(addHeapObject(e));
    }
}
function __wbg_adapter_139(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures__invoke2_mut__h76ed5b8c1aa880a0(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const __wbindgen_enum_ReadableStreamReaderMode = ["byob"];

const FileHeaderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fileheader_free(ptr >>> 0, 1));
/**
 * # FileHeader
 *
 * This type contains necessary fields for both "local file header" and
 * "central directory header" because they have many similar fields.
 */
export class FileHeader {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FileHeaderFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fileheader_free(ptr, 0);
    }
}

const HandlesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_handles_free(ptr >>> 0, 1));

export class Handles {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Handles.prototype);
        obj.__wbg_ptr = ptr;
        HandlesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        HandlesFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_handles_free(ptr, 0);
    }
    /**
     * @param {Array<any>} entries
     * @returns {Promise<any>}
     */
    scan(entries) {
        const ret = wasm.handles_scan(this.__wbg_ptr, addHeapObject(entries));
        return takeObject(ret);
    }
    /**
     * @param {string} output_file_name
     * @param {number} compression_level
     * @returns {Promise<any>}
     */
    compress(output_file_name, compression_level) {
        const ptr0 = passStringToWasm0(output_file_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handles_compress(this.__wbg_ptr, ptr0, len0, compression_level);
        return takeObject(ret);
    }
    /**
     * @param {string} output_file_name
     * @param {number} compression_level
     * @param {string} transform_script
     * @returns {Promise<any>}
     */
    compress_transform(output_file_name, compression_level, transform_script) {
        const ptr0 = passStringToWasm0(output_file_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(transform_script, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.handles_compress_transform(this.__wbg_ptr, ptr0, len0, compression_level, ptr1, len1);
        return takeObject(ret);
    }
    /**
     * @param {string} _transform_script
     * @returns {Promise<any>}
     */
    transform(_transform_script) {
        const ptr0 = passStringToWasm0(_transform_script, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handles_transform(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * @param {string} _transform_script
     * @returns {Promise<any>}
     */
    recover(_transform_script) {
        const ptr0 = passStringToWasm0(_transform_script, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.handles_recover(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
    /**
     * @param {Function} callback
     */
    register_scan_progress(callback) {
        wasm.handles_register_scan_progress(this.__wbg_ptr, addHeapObject(callback));
    }
    /**
     * @param {Function} callback
     */
    register_compress_progress(callback) {
        wasm.handles_register_compress_progress(this.__wbg_ptr, addHeapObject(callback));
    }
    /**
     * @param {Function} callback
     */
    register_average_speed(callback) {
        wasm.handles_register_average_speed(this.__wbg_ptr, addHeapObject(callback));
    }
    /**
     * @param {Function} callback
     */
    register_current_speed(callback) {
        wasm.handles_register_current_speed(this.__wbg_ptr, addHeapObject(callback));
    }
    /**
     * @param {Function} callback
     */
    register_current_file(callback) {
        wasm.handles_register_current_file(this.__wbg_ptr, addHeapObject(callback));
    }
}

const WasmErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmerror_free(ptr >>> 0, 1));

export class WasmError {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmError.prototype);
        obj.__wbg_ptr = ptr;
        WasmErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmErrorFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmerror_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get code() {
        const ret = wasm.__wbg_get_wasmerror_code(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set code(arg0) {
        wasm.__wbg_set_wasmerror_code(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string}
     */
    get arg0() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_wasmerror_arg0(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set arg0(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmerror_arg0(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get arg1() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_wasmerror_arg1(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set arg1(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmerror_arg1(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get arg2() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_wasmerror_arg2(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set arg2(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmerror_arg2(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {string}
     */
    get arg3() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.__wbg_get_wasmerror_arg3(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set arg3(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_wasmerror_arg3(this.__wbg_ptr, ptr0, len0);
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbindgen_object_drop_ref = function(arg0) {
        takeObject(arg0);
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = getObject(arg0) === getObject(arg1);
        return ret;
    };
    imports.wbg.__wbindgen_object_clone_ref = function(arg0) {
        const ret = getObject(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_value_603c13fb803ff74a = function(arg0) {
        const ret = getObject(arg0).value;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_done_205519afd4499873 = function(arg0) {
        const ret = getObject(arg0).done;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_wasmerror_new = function(arg0) {
        const ret = WasmError.__wrap(arg0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = takeObject(arg0).original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_c5419c06eab41e73 = function(arg0) {
        queueMicrotask(getObject(arg0));
    };
    imports.wbg.__wbg_queueMicrotask_848aa4969108a57e = function(arg0) {
        const ret = getObject(arg0).queueMicrotask;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(getObject(arg0)) === 'function';
        return ret;
    };
    imports.wbg.__wbg_new_abda76e883ba8a5f = function() {
        const ret = new Error();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_stack_658279fe44541cf6 = function(arg0, arg1) {
        const ret = getObject(arg1).stack;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_error_f851667af71bcfc6 = function(arg0, arg1) {
        let deferred0_0;
        let deferred0_1;
        try {
            deferred0_0 = arg0;
            deferred0_1 = arg1;
            console.error(getStringFromWasm0(arg0, arg1));
        } finally {
            wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
        }
    };
    imports.wbg.__wbg_instanceof_Window_6575cd7f1322f82f = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_performance_8efa15a3e0d18099 = function(arg0) {
        const ret = getObject(arg0).performance;
        return isLikeNone(ret) ? 0 : addHeapObject(ret);
    };
    imports.wbg.__wbg_root_ef35fdac933ce87b = function(arg0) {
        const ret = getObject(arg0).root;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_now_d3cbc9581625f686 = function(arg0) {
        const ret = getObject(arg0).now();
        return ret;
    };
    imports.wbg.__wbg_instanceof_ReadableStreamByobReader_fe4beab5d38e18f4 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof ReadableStreamBYOBReader;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_read_f5f9c29f470e741d = function(arg0, arg1) {
        const ret = getObject(arg0).read(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getReader_6c4a2f1b4baccac5 = function(arg0, arg1) {
        const ret = getObject(arg0).getReader(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_setmode_9cad09a7356285ff = function(arg0, arg1) {
        getObject(arg0).mode = __wbindgen_enum_ReadableStreamReaderMode[arg1];
    };
    imports.wbg.__wbg_name_e30efb33291e0016 = function(arg0, arg1) {
        const ret = getObject(arg1).name;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_isFile_2a84a01846df768d = function(arg0) {
        const ret = getObject(arg0).isFile;
        return ret;
    };
    imports.wbg.__wbg_isDirectory_fbef237c0907e5f2 = function(arg0) {
        const ret = getObject(arg0).isDirectory;
        return ret;
    };
    imports.wbg.__wbg_fullPath_172b321bc42a5e52 = function(arg0, arg1) {
        const ret = getObject(arg1).fullPath;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_filesystem_1a5137ce8df74932 = function(arg0) {
        const ret = getObject(arg0).filesystem;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_file_7e636ba1ca273fea = function(arg0, arg1, arg2) {
        getObject(arg0).file(getObject(arg1), getObject(arg2));
    };
    imports.wbg.__wbg_instanceof_WritableStreamDefaultWriter_445f6c5d6bdb0f88 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof WritableStreamDefaultWriter;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_ready_b946f0c6dacc9233 = function(arg0) {
        const ret = getObject(arg0).ready;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_close_949410ec7b6cba9f = function(arg0) {
        const ret = getObject(arg0).close();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_write_8516d1ea4c89b39d = function(arg0, arg1) {
        const ret = getObject(arg0).write(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_instanceof_Blob_702ee3ea790162e1 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Blob;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_stream_3780b9c274095bd8 = function(arg0) {
        const ret = getObject(arg0).stream();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_createReader_5f769c2c35d1a395 = function(arg0) {
        const ret = getObject(arg0).createReader();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_getFile_ff461f9adb5e631c = function(arg0, arg1, arg2, arg3, arg4, arg5) {
        getObject(arg0).getFile(arg1 === 0 ? undefined : getStringFromWasm0(arg1, arg2), getObject(arg3), getObject(arg4), getObject(arg5));
    };
    imports.wbg.__wbg_readEntries_30d5ee31e3da5d0b = function() { return handleError(function (arg0, arg1, arg2) {
        getObject(arg0).readEntries(getObject(arg1), getObject(arg2));
    }, arguments) };
    imports.wbg.__wbg_get_5419cf6b954aa11d = function(arg0, arg1) {
        const ret = getObject(arg0)[arg1 >>> 0];
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_length_f217bbbf7e8e4df4 = function(arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_new_034f913e7636e987 = function() {
        const ret = new Array();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newnoargs_1ede4bf2ebbaaf43 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_call_a9ef466721e824f2 = function() { return handleError(function (arg0, arg1) {
        const ret = getObject(arg0).call(getObject(arg1));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_e69b5f66fda8f13c = function() {
        const ret = new Object();
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_self_bf91bf94d9e04084 = function() { return handleError(function () {
        const ret = self.self;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_window_52dd9f07d03fd5f8 = function() { return handleError(function () {
        const ret = window.window;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_globalThis_05c129bf37fcf1be = function() { return handleError(function () {
        const ret = globalThis.globalThis;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_global_3eca19bb09e9c484 = function() { return handleError(function () {
        const ret = global.global;
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = getObject(arg0) === undefined;
        return ret;
    };
    imports.wbg.__wbg_concat_dfce0bd18b227d8c = function(arg0, arg1) {
        const ret = getObject(arg0).concat(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_d6669feb58d6f9fe = function(arg0) {
        const ret = new ArrayBuffer(arg0 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_byteLength_966b6c1f1e5344ae = function(arg0) {
        const ret = getObject(arg0).byteLength;
        return ret;
    };
    imports.wbg.__wbg_call_3bfa248576352471 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_call_5fb7c8066a4a4825 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = getObject(arg0).call(getObject(arg1), getObject(arg2), getObject(arg3));
        return addHeapObject(ret);
    }, arguments) };
    imports.wbg.__wbg_new_1073970097e5a420 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_139(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            const ret = new Promise(cb0);
            return addHeapObject(ret);
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_resolve_0aad7c1484731c99 = function(arg0) {
        const ret = Promise.resolve(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_748f75edfb032440 = function(arg0, arg1) {
        const ret = getObject(arg0).then(getObject(arg1));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_then_4866a7d9f55d8f3e = function(arg0, arg1, arg2) {
        const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_buffer_ccaed51a635d8a2d = function(arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_7e3eb787208af730 = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_new_fec2611eb9180f95 = function(arg0) {
        const ret = new Uint8Array(getObject(arg0));
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_set_ec2fcf81bc573fd9 = function(arg0, arg1, arg2) {
        getObject(arg0).set(getObject(arg1), arg2 >>> 0);
    };
    imports.wbg.__wbg_length_9254c4bd3b9f23c4 = function(arg0) {
        const ret = getObject(arg0).length;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Uint8Array_df0761410414ef36 = function(arg0) {
        let result;
        try {
            result = getObject(arg0) instanceof Uint8Array;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_buffer_95102df5554646dc = function(arg0) {
        const ret = getObject(arg0).buffer;
        return addHeapObject(ret);
    };
    imports.wbg.__wbg_byteLength_5d623ba3d92a3a9c = function(arg0) {
        const ret = getObject(arg0).byteLength;
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(getObject(arg1));
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper254 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 48, __wbg_adapter_24);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper256 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 48, __wbg_adapter_24);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper258 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 48, __wbg_adapter_24);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper259 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 48, __wbg_adapter_24);
        return addHeapObject(ret);
    };
    imports.wbg.__wbindgen_closure_wrapper324 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 58, __wbg_adapter_33);
        return addHeapObject(ret);
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;



    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('wasm_zip_stream_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
