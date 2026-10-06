pragma Singleton
import QtQuick

// Nomes do usuário são chaves, inclusive __proto__ e constructor.
QtObject {
    function copy(source, patch) {
        return Object.assign(Object.create(null), source || {}, patch || {});
    }
    function get(source, name) {
        return source && Object.prototype.hasOwnProperty.call(source, name) ? source[name] : undefined;
    }
}
