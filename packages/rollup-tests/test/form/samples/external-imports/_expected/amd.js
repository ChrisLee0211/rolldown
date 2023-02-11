define(['factory', 'baz', 'shipping-port', 'alphabet'], (function (factory, baz, containers, alphabet) { 'use strict';

	function _interopNamespaceDefault(e) {
		var n = Object.create(null);
		if (e) {
			Object.keys(e).forEach(function (k) {
				if (k !== 'default') {
					var d = Object.getOwnPropertyDescriptor(e, k);
					Object.defineProperty(n, k, d.get ? d : {
						enumerable: true,
						get: function () { return e[k]; }
					});
				}
			});
		}
		n.default = e;
		return Object.freeze(n);
	}

	var containers__namespace = /*#__PURE__*/_interopNamespaceDefault(containers);

	factory( null );
	baz.foo( baz.bar, containers.port );
	containers__namespace.forEach( console.log, console );
	console.log( alphabet.a );
	console.log( alphabet.length );

}));
