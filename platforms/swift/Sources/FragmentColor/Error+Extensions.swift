//
//  Error+Extensions.swift
//  FragmentColor
//
//  The uniffi wrapper gives `FragmentColorError` a `description` from the
//  Rust `Display`, the full text of each case, but no conformance that uses it.
//  With `CustomStringConvertible`, string interpolation and the generated
//  `errorDescription`, and so `localizedDescription`, return that text.
//

import Foundation

extension FragmentColorError: CustomStringConvertible {}
