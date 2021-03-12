var soljson = Module;

var version = soljson.cwrap('solidity_version', 'string', []);

var license = soljson.cwrap('solidity_license', 'string', []);

// >5.0.0
if ('_compileStandard' in soljson) {
    var compile = soljson.cwrap('compileStandard', 'string', ['string', 'number']);
} else {
    var compile = soljson.cwrap('solidity_compile', 'string', ['string', 'number']);
}

var addFunction = soljson.addFunction || soljson.Runtime.addFunction;

var alloc;
if ('_solidity_alloc' in soljson) {
    alloc = soljson.cwrap('solidity_alloc', 'number', ['number']);
} else {
    alloc = soljson._malloc;
    assert(alloc, 'Expected malloc to be present.');
}

var copyToCString = function (str, ptr) {
    var length = soljson.lengthBytesUTF8(str);
    // This is allocating memory using solc's allocator.
    //
    // Before 0.6.0:
    //   Assuming copyToCString is only used in the context of wrapCallback, solc will free these pointers.
    //   See https://github.com/ethereum/solidity/blob/v0.5.13/libsolc/libsolc.h#L37-L40
    //
    // After 0.6.0:
    //   The duty is on solc-js to free these pointers. We accomplish that by calling `reset` at the end.
    var buffer = alloc(length + 1);
    soljson.stringToUTF8(str, buffer, length + 1);
    soljson.setValue(ptr, buffer, '*');
};

var copyFromCString = soljson.UTF8ToString || soljson.Pointer_stringify;

// < 6.0.0
// function _callback(data, contents, error) {}
// >= 6.0.0
// function _callback(context, kind, data, contents, error) {}

// import resolver callback
function _callback(data, contents, error) {
    // log("in callback");
    const importPath = copyFromCString(data);
    // log(importPath);
    // log(copyFromCString(contents));
    // log(copyFromCString(error));
    const source = resolveImport(importPath);

    copyToCString(source, contents);
}

var importCallback = addFunction(_callback, 'viiiii');

log("soljson init ok!");
