define(['exports', './dep'], (function (exports, dep) { 'use strict';



	exports.bar = dep.foo;
	exports.foo = dep.foo;

}));
